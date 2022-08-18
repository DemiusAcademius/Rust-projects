extern crate libc;

use super::oci;
use super::conn;
use super::bindings;

pub struct Statement<'a>  {
    pub conn:         &'a conn::Connection,
    pub stmt_handle:  *mut oci::OCIStmt,
    pub error_handle: *mut oci::OCIError,
    stmt_hash:    String
}

impl<'a> Statement<'a> {
    pub fn new(conn: &'a conn::Connection, stmt_text: String, bindmap: Option<bindings::Bindmap>) -> Result<Statement<'a>, oci::OracleError> {
        let stmt_hash = stmt_text.clone(); // hashing is currently unstable
        let error_handle = conn.env.error_handle;
        let stmt_handle = oci::oci_stmt_prepare2(conn.service_handle, error_handle, &stmt_text, &stmt_hash)?;

        if let Some(bindmap) = bindmap {
            for (placeholder, b) in bindmap.iter() {
                let b = b.borrow();
                oci::oci_stmt_bind_by_name(stmt_handle, error_handle,
                    &placeholder.to_string(),
                    b.value_p, b.indic_p,
                    b.max_size as usize, &b.actual_size,
                    b.oci_type.to_owned())?
            }
        }
        
        Ok(Statement { conn, stmt_handle, error_handle, stmt_hash })
    }

    pub fn execute(&self, iterations: u32) -> Result<(), oci::OracleError> {
        oci::oci_stmt_execute(self.conn.service_handle, self.stmt_handle, self.error_handle, iterations)?;
        Ok(())
    }
    
}

impl<'a> Drop for Statement<'a> {
    fn drop(&mut self) {
        oci::oci_stmt_release(self.stmt_handle, self.error_handle, &self.stmt_hash).
            ok().expect("oci_stmt_release failed");
    }
}