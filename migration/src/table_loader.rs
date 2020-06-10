extern crate libc;

use std::cmp;
use std::mem;
use std::ptr;
use std::slice;

use table;
use table::ColumnType;

use oracle::oci;
use oracle::Connection;
use oracle::{ LobDescriptor, lob_copy };
use oracle::OCITempLobType;

pub struct TableReader {
    stmt_handle:  *mut oci::OCIStmt,
    error_handle: *mut oci::OCIError,
    stmt_hash:     String
}

pub struct TableWriter {
    stmt_handle:  *mut oci::OCIStmt,
    error_handle: *mut oci::OCIError,
    stmt_hash:     String    
}

pub struct LoadBuffer {
    values_size:      u32,
    indicators_size:  u32,

    values_p:      *const u8,               // pointer to values area
    indicators_p:  *const libc::c_short,    // pointer to indicators area
    ret_lengths_p: *const libc::c_ushort,    // pointer to return lengths area    
}

pub struct LobProcessor<'a> {
    // pairs of lob locators -> src, dst
    lob_locators: Vec<(LobDescriptor<'a>, LobDescriptor<'a>)>
}

impl TableReader {
    pub fn new(conn: &Connection, stmt_text: String) -> Result<TableReader, oci::OracleError> {
        let stmt_hash = stmt_text.clone(); // hashing is currently unstable
        let error_handle = conn.env.error_handle;
        let stmt_handle = oci::oci_stmt_prepare2(conn.service_handle, error_handle, &stmt_text, &stmt_hash)?;

        Ok(TableReader { stmt_handle, error_handle, stmt_hash })
    }

    pub fn execute(&mut self, conn: &Connection) -> Result<(), oci::OracleError> {
        oci::oci_stmt_execute(conn.service_handle, self.stmt_handle, self.error_handle, 0)
    }    

    pub fn fetch(&mut self, prefetch_rows: u32) -> Result<(u32, bool), oci::OracleError> {
        let mut done = false;

        if let Err(error) = 
                oci::oci_stmt_fetch(self.stmt_handle,
                                    self.error_handle,
                                    prefetch_rows,
                                    oci::OCIOrientation::FetchNext, 0) {

            if error.code == 100 {
                done = true;
            } else {
                return Err(error);
            }

        }

        let mut rows_fetched: u32 = 0;
        let rows_fetched_ptr = unsafe { mem::transmute::<&mut u32, *mut oci::c_void>(&mut rows_fetched) };

        oci::oci_attr_get(self.stmt_handle as *mut oci::c_void,
                               oci::OCIHandleType::Statement, 
                               rows_fetched_ptr,
                               oci::OCIAttribute::RowsFetched,
                               self.error_handle)?;

        Ok((rows_fetched, done))
    }
    
}

impl Drop for TableReader {
    fn drop(&mut self) {
        oci::oci_stmt_release(self.stmt_handle, self.error_handle, &self.stmt_hash).
            ok().expect("oci_stmt_release failed");
    }
}

impl TableWriter {
    pub fn new(conn: &Connection, stmt_text: String) -> Result<TableWriter, oci::OracleError> {
        let stmt_hash = stmt_text.clone(); // hashing is currently unstable
        let error_handle = conn.env.error_handle;
        let stmt_handle = oci::oci_stmt_prepare2(conn.service_handle, error_handle, &stmt_text, &stmt_hash)?;

        Ok(TableWriter { stmt_handle, error_handle, stmt_hash })
    }

    pub fn execute(&mut self, conn: &Connection, rows_num: u32) -> Result<(), oci::OracleError> {
        oci::oci_stmt_execute(conn.service_handle, self.stmt_handle, self.error_handle, rows_num)
    }    
    
}

impl Drop for TableWriter {
    fn drop(&mut self) {
        oci::oci_stmt_release(self.stmt_handle, self.error_handle, &self.stmt_hash).
            ok().expect("oci_stmt_release failed");
    }
}

impl LoadBuffer {
    pub fn new(buffer_size: u32) -> LoadBuffer {
        let values_size = buffer_size;
        let indicators_size = buffer_size / 16;

        let values_p = unsafe { libc::malloc(values_size as libc::size_t) as *const u8 };
        let indicators_p = unsafe { libc::malloc(indicators_size as libc::size_t*2) as *const libc::c_short };
        let ret_lengths_p = unsafe { libc::malloc(indicators_size as libc::size_t*2) as *const libc::c_ushort }; 

        if values_p.is_null() || indicators_p.is_null() || ret_lengths_p.is_null() {
            panic!("failed to allocate mem for Cursor");
        }

        println!("buffer size: {}, indicators_size: {}", buffer_size, indicators_size);
        
        LoadBuffer { values_size, indicators_size, values_p, indicators_p, ret_lengths_p }
    }

    fn get_prefetch_rows(&self, table_info: &table::TableInfo) -> u32 {
        if table_info.has_blob {
            1
        } else {
            let ref columns_info = table_info.columns;
            let columns_cnt = columns_info.len();
            let max_indic_rows = self.indicators_size as usize / columns_cnt - 1;

            let mut row_size: usize = 0;

            for c in columns_info.iter() {
                row_size += c.buffer_len as usize;
            }

            let max_area_rows = self.values_size as usize / row_size - 1;

            cmp::min(max_indic_rows, max_area_rows) as u32
        }
    }

    pub fn bind<'a>(&self, table_info: &table::TableInfo,
                    source: &'a Connection,
                    destination: &'a Connection,  
                    reader: &TableReader, writer: &TableWriter)-> Result<(u32, LobProcessor<'a>), oci::OracleError> {
        let reader_stmt_handle = reader.stmt_handle;
        let reader_error_handle = reader.error_handle;

        let writer_stmt_handle = writer.stmt_handle;
        let writer_error_handle = writer.error_handle;

        // let prefetch_rows = if table_info.name.contains("APEDUCT_") { 100 } else { self.get_prefetch_rows(table_info) };
        let prefetch_rows = self.get_prefetch_rows(table_info);

        let mut lob_locators: Vec<(LobDescriptor<'a>,LobDescriptor<'a>)> = Vec::new();

        let charset_id = oci::OCICharset::EE8ISO8859P2 as u16;
        let charset_id_ptr = unsafe { mem::transmute::<&u16, *mut oci::c_void>(&charset_id) };

        unsafe {
            let value = self.values_size / 8;   
            let value_ptr = mem::transmute::<&u32, *mut oci::c_void>(&value);
            oci::oci_attr_set(reader_stmt_handle as *mut oci::c_void,
                                oci::OCIHandleType::Statement,
                                value_ptr, 4,
                                oci::OCIAttribute::PrefetchMemory,
                                reader_error_handle)?;
            
            let mut offset = 0;
            let mut offset_i = 0;
            let mut offset_s = 0;
            for (i, m) in table_info.columns.iter().enumerate() {

                let ind_p: *const libc::c_short = self.indicators_p.offset(offset_i);
                offset_i += prefetch_rows as isize;

                if m.col_type != ColumnType::Clob && m.col_type != ColumnType::Blob {
                    let value_p: *const u8 = self.values_p.offset(offset);

                    let size_p: *const libc::c_ushort = self.ret_lengths_p.offset(offset_s);

                    offset += (m.buffer_len * prefetch_rows) as isize;
                    offset_s += prefetch_rows as isize;

                    oci::oci_stmt_define_by_pos(reader_stmt_handle, reader_error_handle,
                        i + 1, value_p, ind_p, m.buffer_len as isize, size_p, m.oci_data_type.to_owned())?;
                        
                    let bind_handle = oci::oci_stmt_bind_by_pos(writer_stmt_handle, writer_error_handle,
                        i + 1, value_p, ind_p, m.buffer_len as isize, size_p, m.oci_data_type.to_owned())?;

                    if m.col_type == ColumnType::Varchar || m.col_type == ColumnType::Long { 
                        oci::oci_attr_set(bind_handle as *mut oci::c_void,
                                          oci::OCIHandleType::Bind,
                                          charset_id_ptr as *mut oci::c_void,
                                          0,
                                          oci::OCIAttribute::CharsetId,
                                          writer_error_handle)?;
                    }
                        
                } else {
                    let lob_type = 
                        if m.col_type == ColumnType::Clob { OCITempLobType::Clob } else { OCITempLobType::Blob };

                    let src_lob = LobDescriptor::new(source, false, OCITempLobType::Default, ind_p)?;
                    let dst_lob = LobDescriptor::new(destination, true, lob_type, ind_p)?;

                    let src_locator_ptr: *const u8 = mem::transmute( &(*src_lob.locator) );
                    let dst_locator_ptr: *const u8 = mem::transmute( &(*dst_lob.locator) );

                    oci::oci_stmt_define_by_pos(reader_stmt_handle, reader_error_handle,
                        i + 1, src_locator_ptr, ind_p, -1, ptr::null(), m.oci_data_type.to_owned())?;
                        
                    oci::oci_stmt_bind_by_pos(writer_stmt_handle, writer_error_handle,
                        i + 1, dst_locator_ptr, ind_p, -1, ptr::null(), m.oci_data_type.to_owned())?;

                    lob_locators.push((src_lob, dst_lob));
                }
            }
        }
        Ok( (prefetch_rows, LobProcessor { lob_locators }) )
    }

    pub fn check_lengths(&self, table_info: &table::TableInfo, prefetch_rows: u32, actual_rows: u32) {
        unsafe {
            let mut offset_i = 0;
            let mut offset_s = 0;
            for (i, m) in table_info.columns.iter().enumerate() {
                if m.col_type != ColumnType::Clob && m.col_type != ColumnType::Blob {
                    for row in 0..actual_rows {
                        let ind_p: *const libc::c_short = self.indicators_p.offset(offset_i + row as isize);

                        if m.col_type == ColumnType::Varchar {
                            let row_indicator = *ind_p;
                            let is_null = row_indicator < 0;
                            if is_null {
                                // null is null
                            } else {
                                let size_p: *const libc::c_ushort = self.ret_lengths_p.offset(offset_s + row as isize);
                                let size = *size_p;
                                if size > m.buffer_len as u16 {
                                    println!("actual len {} of column {} > declared len {}, row {}", size, m.name, m.buffer_len, row);
                                }
                            }
                        } // varchar

                    } // iterace over rows
                    offset_s += prefetch_rows as isize;
                } // only non-blobs
                offset_i += prefetch_rows as isize;
            } // iterate over columns
        }        
    }

    pub fn trans_rom_utf(&self, table_info: &table::TableInfo, prefetch_rows: u32, actual_rows: u32) {
        for row in 0..actual_rows {
            self.trans_rom_utf_row(table_info, prefetch_rows, row as isize);     
        } 
    }

    fn trans_rom_utf_row(&self, table_info: &table::TableInfo, prefetch_rows: u32, index: isize) {
        let mut v_offset: isize = 0;
        let mut i_offset: isize = index;
        let mut s_offset: isize = index;
        unsafe {
            for (i, m) in table_info.columns.iter().enumerate() {
                if m.col_type != ColumnType::Clob && m.col_type != ColumnType::Blob { 
                    if m.col_type == ColumnType::Varchar { 
                        let offset = v_offset + (m.buffer_len as isize * index); 
                        let value_p: *const u8 = self.values_p.offset(offset);

                        let row_indicator = *self.indicators_p.offset(i_offset);
                        let is_not_null = row_indicator >= 0;

                        if is_not_null {
                            let len = *self.ret_lengths_p.offset(s_offset);                            
                            let bytes = slice::from_raw_parts(value_p, len as usize);
                            for b in bytes {
                                match *b {
                                    186u8 => println!("litera s mica"),
                                    170u8 => println!("litera S mare"),
                                    227u8 => println!("litera a mare"),
                                    195u8 => println!("litera A mare"),
                                    _ => {}
                                }
                            }  
                        }
                    }
                    s_offset += prefetch_rows as isize;
                    v_offset += (m.buffer_len * prefetch_rows) as isize;
                }
                i_offset += prefetch_rows as isize;
            }            
        }        
    }


}


impl Drop for LoadBuffer {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.values_p as *mut libc::c_void);
            libc::free(self.indicators_p as *mut libc::c_void);
            libc::free(self.ret_lengths_p as *mut libc::c_void);
        }
    }
}

impl <'a> LobProcessor<'a> {

    pub fn copy(&mut self, buffer: &[u8]) -> Result<(), oci::OracleError> {
        let ref mut locators = self.lob_locators;
 
        for ref mut lob_pair in locators {            
            if lob_pair.0.is_null() {
                lob_pair.1.trim(0)?;
            } else {
                let l = lob_copy(&mut lob_pair.0, &mut lob_pair.1, buffer)?;
            }
        }
        Ok(())
    }

}