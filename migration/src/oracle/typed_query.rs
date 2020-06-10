use super::oci;
use super::conn;
use super::meta::*;
use super::query::*;
use super::bindings;

pub struct QueryBuilder<'a> {
    conn:          &'a conn::Connection,
    stmt_text:     String,
    // bindmap:       bindings::Bindmap<'a>,
    bindmap:       Option<bindings::Bindmap<'a>>,
    prefetch_rows: u32
}

pub struct TypedQuery<'a, V: MetaQuery>  {
    creator: fn(&ResultSet) -> V,
    query:   Query<'a>   
}

pub struct TypedQueryIterator<'iter, 'q: 'iter, V: MetaQuery> {
    creator:  fn(&ResultSet) -> V,
    iterator: QueryIterator<'iter, 'q>
}

impl<'a> QueryBuilder<'a> {

    pub fn new(conn: &'a conn::Connection, stmt_text: String) -> QueryBuilder<'a> {
        // QueryBuilder { conn: conn, stmt_text: stmt_text, prefetch_rows: 10 }
        QueryBuilder { conn: conn, stmt_text: stmt_text, bindmap: None, prefetch_rows: 10 }
    }

    /*
    pub fn bind<B>(mut self, placeholder: &'a str, binding: B) -> Self
            where B: Into<bindings::RowBinding> {
        self.bindmap.insert(placeholder, binding.into());
        self
    }
    */

    pub fn bind(mut self, bindmap: Option<bindings::Bindmap<'a>>) -> QueryBuilder<'a> {
        self.bindmap = bindmap;
        self
    }

    pub fn prefetch(mut self, rows: u32) -> QueryBuilder<'a> {
        if rows < 1 {
            panic!("prefetch_rows MUST have at least 1");
        }
        self.prefetch_rows = rows;
        self
    }

    pub fn prepare<V: MetaQuery>(self) -> Result<TypedQuery<'a, V>, oci::OracleError> {
        TypedQuery::new(self.conn, self.stmt_text, self.prefetch_rows, self.bindmap)
    }

    /*
    pub fn fetch<V: MetaQuery>(self) -> Result<Option<V>, oci::OracleError> {
        let mut q = TypedQuery::new(self.conn, self.stmt_text, 1)?;
        q.fetch(Some(self.bindmap))
    }

    pub fn fetch_vec<V: MetaQuery>(self) -> Result<Vec<V>, oci::OracleError> {
        let mut q = TypedQuery::new(self.conn, self.stmt_text, self.prefetch_rows)?;
        q.fetch_vec(Some(self.bindmap))
    }
    */

    /*
    pub fn fetch_iter<V: MetaQuery>(self) -> Result<TypedQueryIterator<'a, V>, oci::OracleError> {
        let mut q = TypedQuery::new(self.conn, self.stmt_text, self.prefetch_rows)?;
        TypedQueryIterator::new(q, Some(self.bindmap))
    }
    */
     
} 

impl<'a,V: MetaQuery> TypedQuery<'a, V> {

    pub fn new(conn: &'a conn::Connection, stmt_text: String, prefetch_rows: u32, bindmap: Option<bindings::Bindmap>) -> 
            Result<TypedQuery<'a, V>, oci::OracleError> {
        let creator = V::create;
        let query = Query::new(conn, stmt_text, prefetch_rows, V::meta(), bindmap)?;
        Ok(TypedQuery { creator, query })        
    }

    pub fn fetch(&mut self) -> Result<Option<V>, oci::OracleError> {
        let creator = self.creator;
        self.query.one(None, |rs| Some((creator)(&rs)))
    }
        
    pub fn fetch_vec(&mut self) -> Result<Vec<V>, oci::OracleError> {
        let creator = self.creator;
        self.query.fold(Vec::new() as Vec<V>, |mut v, rs| {
            v.push((creator)(&rs));
            v
        })
    }

    pub fn for_each<F>(&mut self, mut f: F) -> Result<(), oci::OracleError>
            where F: FnMut(V) {
        let creator = self.creator;
        self.query.for_each(|rs| f((creator)(&rs)))
    }

    pub fn fold<B,F>(&mut self, init: B, mut f: F) -> Result<B, oci::OracleError> 
            where F: FnMut(B, V) -> B {
        let creator = self.creator;
        self.query.fold(init, |a, rs| f(a, (creator)(&rs)))
    }

    pub fn iterator<'iter>(&'iter mut self) -> Result<TypedQueryIterator<'iter, 'a, V>, oci::OracleError> {
        let creator = self.creator;
        self.query.iterator().and_then(|iterator| Ok(TypedQueryIterator::new(creator, iterator)) )
    }
    
}

impl<'iter, 'q: 'iter, V: MetaQuery> TypedQueryIterator<'iter, 'q, V> {

    pub fn new(creator:  fn(&ResultSet) -> V, iterator: QueryIterator<'iter, 'q>) -> TypedQueryIterator<'iter, 'q, V> {
        TypedQueryIterator { creator, iterator }
    }

}

impl <'iter, 'q: 'iter, V: MetaQuery> Iterator for TypedQueryIterator<'iter, 'q, V> {
    type Item = V;

    fn next(&mut self) -> Option<V> {
        match self.iterator.next() {
            None => None,
            Some(result) => Some((self.creator)(&result)) 
        }
    }
}