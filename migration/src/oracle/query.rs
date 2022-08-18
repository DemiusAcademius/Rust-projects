use super::libc;

use std::mem;

use super::oci;
use super::conn;
use super::stmt;
use super::meta::{ MetaType, ResultSet, ResultItem };
use super::bindings;

pub struct Query<'a> {
    statement:  stmt::Statement<'a>,    
    fetch_area: FetchArea
}

/// struct of area is: col0 ... colN
/// each of columns is vector of rows (prfetch_rows)
struct FetchArea {
    prefetch_rows: u32,
    meta:          Vec<MetaType>,

    values_p:      *const u8,               // pointer to values area
    indicators_p:  *const libc::c_short,    // pointer to indicators area
    ret_lengths_p: *const libc::c_ushort,    // pointer to return lengths area
}

pub struct QueryIterator<'iter, 'q: 'iter> {
    query:        &'iter mut Query<'q>,
    done:         bool,
    rows_fetched: u32,
    cursor_index: u32,
}

impl<'a> Query<'a> {

    pub fn new(conn: &'a conn::Connection, stmt_text: String, prefetch_rows: u32, meta: Vec<MetaType>, bindmap: Option<bindings::Bindmap>)
             -> Result<Query<'a>, oci::OracleError> {
        let statement = stmt::Statement::new(conn, stmt_text, bindmap)?;
        let fetch_area = FetchArea::new(&statement, prefetch_rows, meta)?;
            
        Ok(Query{ statement, fetch_area })
    }

    pub fn iterator<'iter>(&'iter mut self) -> Result<QueryIterator<'iter, 'a>, oci::OracleError> {
        QueryIterator::new(self)
    }

    pub fn for_each<F>(&mut self, mut f: F) -> Result<(), oci::OracleError> 
        where F: FnMut(&ResultSet) {
        let iterator = self.iterator()?;
        for result in iterator {
            f(&result);    
        }   

        Ok(())
    }

    pub fn fold<B,F>(&mut self, init: B, mut f: F) -> Result<B, oci::OracleError> 
            where F: FnMut(B, &ResultSet) ->B {
        let mut acumulator = init;

        let iterator = self.iterator()?;
        for result in iterator {
            acumulator = f(acumulator, &result);    
        }   

        Ok(acumulator)
    }    

    pub fn one<B,F>(&mut self, init: B, mut f: F) -> Result<B, oci::OracleError> 
            where F: FnMut(&ResultSet) ->B {
        let mut acumulator = init;

        let mut iterator = self.iterator()?;

        if let Some(result) = iterator.next() {
            acumulator = f(&result);
        }

        Ok(acumulator)
    }    
    
    fn fetch(&mut self) -> Result<(u32, bool), oci::OracleError> {
        let mut done = false;

        if let Err(error) = 
                oci::oci_stmt_fetch(self.statement.stmt_handle,
                                    self.statement.error_handle,
                                    self.fetch_area.prefetch_rows,
                                    oci::OCIOrientation::FetchNext, 0) {

            if error.code == 100 {
                done = true;
            } else {
                return Err(error);
            }

        }

        let mut rows_fetched: u32 = 0;
        let rows_fetched_ptr = unsafe { mem::transmute::<&mut u32, *mut oci::c_void>(&mut rows_fetched) };

        oci::oci_attr_get(self.statement.stmt_handle as *mut oci::c_void,
                               oci::OCIHandleType::Statement, 
                               rows_fetched_ptr,
                               oci::OCIAttribute::RowsFetched,
                               self.statement.error_handle)?;

        Ok((rows_fetched, done))
    }
        
}

impl FetchArea {
    pub fn new(statement: &stmt::Statement, prefetch_rows: u32, meta: Vec<MetaType>) -> Result<FetchArea, oci::OracleError> {
        let val_size = meta.iter().map(|m| m.size).sum::<u32>();

        let area_size = val_size * prefetch_rows;
        let inds_size = 2 * meta.len() * prefetch_rows as usize;

        let values_p = unsafe { libc::malloc(area_size as libc::size_t) as *const u8 };
        let indicators_p = unsafe { libc::malloc(inds_size as libc::size_t) as *const libc::c_short };
        let ret_lengths_p = unsafe { libc::malloc(inds_size as libc::size_t) as *const libc::c_ushort }; 

        if values_p.is_null() || indicators_p.is_null() || ret_lengths_p.is_null() {
            panic!("failed to allocate mem for Cursor");
        }

        let stmt_handle = statement.stmt_handle;
        let error_handle = statement.error_handle;

        unsafe {
            let mut offset = 0;
            let mut offset_i = 0;
            for (i, m) in meta.iter().enumerate() {
                let value_p: *const u8 = values_p.offset(offset);

                let ind_p: *const libc::c_short = indicators_p.offset(offset_i);
                let size_p: *const libc::c_ushort = ret_lengths_p.offset(offset_i);

                offset += (m.size * prefetch_rows) as isize;
                offset_i += prefetch_rows as isize;

                oci::oci_stmt_define_by_pos(statement.stmt_handle, statement.error_handle,
                    i + 1, value_p, ind_p, m.size as isize, size_p, m.oci_type.to_owned())?;
            }
        }
            
        Ok(FetchArea{ prefetch_rows, meta, values_p, indicators_p, ret_lengths_p })
    }

    fn result(&mut self, index: isize) -> ResultSet {
        let mut result_vector = Vec::with_capacity(self.meta.len()) as Vec<ResultItem>;

        let mut v_offset: isize = 0;
        let mut i_offset: isize = index;

        unsafe {
            for m in self.meta.iter() {

                let offset = v_offset + (m.size as isize * index); 
                let value_p: *const u8 = self.values_p.offset(offset);

                let row_indicator = *self.indicators_p.offset(i_offset);
                let indicator = row_indicator >= 0;

                let len = *self.ret_lengths_p.offset(i_offset);

                v_offset += (m.size * self.prefetch_rows) as isize;
                i_offset += self.prefetch_rows as isize;

                let result = ResultItem::new(value_p, indicator, len as usize);

                result_vector.push(result);
                
            }
            result_vector
        }        
    }
    
}

impl Drop for FetchArea {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.values_p as *mut libc::c_void);
            libc::free(self.indicators_p as *mut libc::c_void);
            libc::free(self.ret_lengths_p as *mut libc::c_void);
        }
    }
}

impl <'iter, 'q: 'iter> QueryIterator <'iter, 'q> {

    pub fn new(query: &'iter mut Query<'q>) -> Result<QueryIterator<'iter, 'q>, oci::OracleError> {
        {
            let ref mut statement = query.statement;
            statement.execute(0)?;
        }
        Ok( QueryIterator { query: query, done: false, rows_fetched: 0, cursor_index: 0 } )
    }
        
}

impl <'iter, 'q: 'iter> Iterator for QueryIterator <'iter, 'q> {
    type Item = ResultSet;

    fn next(&mut self) -> Option<ResultSet> {
        if self.done && self.cursor_index == 0 {
            return None;
        }

        if self.cursor_index == 0 {
            match self.query.fetch() {
                Ok((rows_fetched,done)) => {
                    self.rows_fetched = rows_fetched;
                    self.done = done;
                }
                Err(_) => return None
            }
        }

        if self.rows_fetched > 0 {
            let ref mut fetch_area = self.query.fetch_area;
            let result = fetch_area.result(self.cursor_index as isize);
            self.cursor_index += 1;

            if self.cursor_index == self.rows_fetched {
                // for next fetch
                self.cursor_index = 0;
            }

            Some(result)
        } else {
            None
        }
                
    }
}