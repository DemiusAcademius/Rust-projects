#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ 
    error, fmt, ptr
 };
 pub use std::ffi::{ c_void, CString };

#[allow(dead_code)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/oci-bindings.rs"));
}
pub use bindings::*;

/// Represents Oracle error 
#[derive(Debug)]
pub struct OracleError {
    /// Oracle error code
    pub errcode: i32,
    /// Message from Oracle
    message:     String,
    // Function where error occured
    location:    &'static str
}

impl fmt::Display for OracleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!{f, "\n\n   Error code: {}\n   Error message: {}\n   Where: {}\n\n",
                self.errcode, self.message, self.location}
    }
}

impl error::Error for OracleError {
    fn description(&self) -> &str {
        self.message.as_str()
    }
}

/// Returns an error message in the buffer provided and an ORACLE error
#[inline]
fn error_get(errhp: *mut OCIError, location: &'static str) -> OracleError {
    let errc: *mut i32 = &mut 0;
    let mut buf = String::with_capacity(2048);
    unsafe {
        OCIErrorGet(
            errhp as *mut c_void, // hndlp
            1,                    // recordno
            ptr::null_mut(),      // sqlstate
            errc,                 // errcodep
            buf.as_mut_ptr() as *mut u8,  // bufp
            buf.capacity() as u32,        // bufsiz
            OCI_HTYPE_ERROR
        )
    };
    OracleError { errcode: unsafe{ *errc }, message: buf, location }
}

/// check errcode for Oracle Error
pub fn check_error(errcode: i32,
                   handle: Option<*mut OCIError>,
                   location: &'static str) -> Result<(), OracleError> {
    if errcode == 0 /* OCI_SUCCESS */ {
        Ok(())
    } else {
        let by_handle = 
            handle.map(|errhp| {
                let mut error = error_get(errhp, location);
                if error.errcode == 24347 {
                    error.message = "NULL column in a aggregate function".to_string();
                }
                error
            });

        let oracleerr = 
            if errcode == -1  /* OCI_ERROR */ {
                by_handle.unwrap_or(
                    OracleError { errcode, message: "Error with no details".to_string(), location }
                )
            } else if errcode == 1 /* OCI_SUCCESS_WITH_INFO */ {
                by_handle.unwrap_or(
                    OracleError { errcode, message: "Success with info".to_string(), location }
                )
                
            } else {
                let message = 
                    match errcode {
                        100 /* OCI_NO_DATA */ => "No data",
                        -2 /* OCI_INVALID_HANDLE */ => "Invalid handle",
                        99 /* OCI_NEED_DATA */ => "Need data",
                        -3123 /* OCI_STILL_EXECUTING */ => "Need data",
                        _ => panic!("Unknow return code")
                    }.to_string();
                OracleError { errcode, message, location }
            };
            Err(oracleerr)
        }        
}

/// creates and initializes an environment for the rest of the OCI functions
#[inline]
pub fn env_create() -> Result<*mut bindings::OCIEnv, OracleError> {
    let mut envhp = ptr::null_mut();

    check_error(
        unsafe {
            OCIEnvCreate(
                &mut envhp,         // envhp
                OCI_DEFAULT,        // mode
                ptr::null_mut(),    // ctxp
                None,               // malocfp
                None,               // ralocfp
                None,               // mfreefp
                0,                  // xtramem_sz
                ptr::null_mut()     // usrmempp
            )
        }, None, "oci::env_create").map(|_| envhp)
} 

/// Returns a pointer to an allocated and initialized handle
#[inline]
pub fn handle_alloc(envhp: *mut OCIEnv, htype: u32)
            -> Result<*mut c_void, OracleError> {
    let mut handle = ptr::null_mut();

    check_error(
        unsafe {
            OCIHandleAlloc(
                envhp as *const _,  // parenth
                &mut handle,        // hndlpp
                htype,              // type_
                0,                  // xtramem_sz
                ptr::null_mut()     // usrmempp
            )
        }, None, "oci::handle_alloc").map(|_| handle)
}

/// Explicitly deallocates a handle
#[inline]
pub fn handle_free(handle: *mut c_void, htype: u32) {
    check_error(
        unsafe { OCIHandleFree(handle, htype) }, None, "oci::handle_free").unwrap();
}

/// Do cleanup before process termination
#[inline]
pub fn terminate() {
    unsafe { OCITerminate(OCI_DEFAULT) };
}

/// used to get a particular attribute of a handle
#[inline]
pub fn attr_get(handle: *mut c_void, 
                htype: u32,
                value: *mut c_void,
                attr_type: u32,
                errhp: *mut OCIError)
            -> Result<u32, OracleError> {
    let mut attr_size = 0;

    check_error(
        unsafe {
            OCIAttrGet(handle as *const _, htype, value, &mut attr_size, attr_type, errhp)
        }, Some(errhp), "oci::attr_get").map(|_| attr_size)
}

/// sed to set a particular attribute of a handle or a descriptor
#[inline]
pub fn attr_set(handle: *mut c_void, 
                htype: u32,
                value: *mut c_void,
                attr_size: u32,
                attr_type: u32,
                errhp: *mut OCIError)
            -> Result<(), OracleError> {
    check_error(
        unsafe {
            OCIAttrSet(handle, htype, value, attr_size, attr_type, errhp)
        }, Some(errhp), "oci::attr_set")
}

/// used to create an association between an OCI application and aparticular server
#[inline]
pub fn server_attach(srvhp: *mut OCIServer, errhp: *mut OCIError, db: &str) -> Result<(), OracleError> {
    let db_len = db.len();
    let db = CString::new(db).unwrap();

    check_error(
        unsafe {
            OCIServerAttach(srvhp, errhp, db.as_ptr() as *const u8, db_len as i32, OCI_DEFAULT)
        }, Some(errhp), "oci::server_attach")
}

/// deletes an access to data source for OCI operations
#[inline]
pub fn server_detach(srvhp: *mut OCIServer, errhp: *mut OCIError) {
    check_error(
        unsafe {
            OCIServerDetach(srvhp, errhp, OCI_DEFAULT)
        }, Some(errhp), "oci::server_detach").unwrap();
}

/// creates a user authentication and begins a user session for a given server
#[inline]
pub fn session_begin(svchp: *mut OCISvcCtx, errhp: *mut OCIError, authp: *mut OCISession)
         -> Result<(), OracleError> {
    check_error(
        unsafe {
            OCISessionBegin(svchp, errhp, authp, OCI_CRED_RDBMS, OCI_DEFAULT)
        }, Some(errhp), "oci::session_begin")
}

/// terminates a user authentication context created by OCISessionBegin()
#[inline]
pub fn session_end(svchp: *mut OCISvcCtx, errhp: *mut OCIError, authp: *mut OCISession) {
    check_error(
        unsafe {
            OCISessionEnd(svchp, errhp, authp, OCI_DEFAULT)
        }, Some(errhp), "oci::session_end").unwrap();
}

/// commit transaction in write nowait mode
#[inline]
pub fn commit(svchp: *mut OCISvcCtx, errhp: *mut OCIError) -> Result<(), OracleError> {
    check_error(
        unsafe {
            OCITransCommit(svchp, errhp, OCI_TRANS_WRITENOWAIT)
        }, Some(errhp), "oci::commit")                
}

/// rollback transaction
#[inline]
pub fn rollback(svchp: *mut OCISvcCtx, errhp: *mut OCIError) -> Result<(), OracleError> {
    check_error(
        unsafe {
            OCITransRollback(svchp, errhp, OCI_DEFAULT)
        }, Some(errhp), "oci::rollback")                
}

/// defines the SQL/PLSQL statement to be executed
#[inline]
pub fn stmt_prepare(svchp: *mut OCISvcCtx, errhp: *mut OCIError, sql: &str) -> Result<*mut OCIStmt, OracleError> {
    let mut handle = ptr::null_mut() as *mut OCIStmt;
    let sql_len = sql.len() as u32;
    let sql = CString::new(sql).unwrap();

    check_error(
        unsafe {
            OCIStmtPrepare2(svchp, &mut handle, errhp, sql.as_ptr() as *const u8, sql_len, ptr::null(), 0, OCI_NTV_SYNTAX, OCI_DEFAULT)
        }, Some(errhp), "oci::stmt_prepare").map(|_| handle)
}

/// release the SQL/PLSQL statement
#[inline]
pub fn stmt_release(stmthp: *mut OCIStmt, errhp: *mut OCIError) {
    check_error(
        unsafe {
            OCIStmtRelease(stmthp, errhp, ptr::null(), 0, OCI_DEFAULT)
        }, Some(errhp), "oci::stmt_release").unwrap();
}

/// associates an application request with a serve
#[inline]
pub fn stmt_execute(svchp: *mut OCISvcCtx, stmthp: *mut OCIStmt, errhp: *mut OCIError, iters: u32, rowoff: u32) -> Result<(), OracleError> {
    check_error(
        unsafe {
            OCIStmtExecute(svchp, stmthp, errhp, iters, rowoff, ptr::null(), ptr::null_mut(), OCI_DEFAULT)
        }, Some(errhp), "oci::stmt_execute")
}

/// defines an output buffer which will receive data retreived from Oracle
#[inline]
pub fn define_by_pos(stmthp: *mut OCIStmt,
                     errhp: *mut OCIError,
                     position: u32,
                     valuep: *mut c_void,
                     indp: *mut c_void,
                     size: i32,
                     rlenp: *mut u16,
                     dtype: u16) -> Result<*mut OCIDefine, OracleError> {
    let mut handle = ptr::null_mut() as *mut OCIDefine;

    check_error(
        unsafe {
            OCIDefineByPos(stmthp, &mut handle, errhp, position, valuep, size, dtype, indp, rlenp, ptr::null_mut(), OCI_DEFAULT)
        }, Some(errhp), "oci::define_by_pos").map(|_| handle)
}

/// creates an association between a program variable and a placeholder in a SQL
#[inline]
pub fn bind_by_pos(stmthp: *mut OCIStmt,
                   errhp: *mut OCIError,
                   position: u32,
                   valuep: *mut c_void,
                   indp: *mut c_void,
                   size: i64,
                   alenp: *mut u32,
                   dtype: u16) -> Result<*mut OCIBind, OracleError> {
    let mut handle = ptr::null_mut() as *mut OCIBind;

    check_error(
        unsafe {
            OCIBindByPos2(stmthp, &mut handle, errhp, position, valuep, size, dtype, indp, alenp, ptr::null_mut(), 1, ptr::null_mut(), OCI_DEFAULT)
        }, Some(errhp), "oci::bind_by_pos").map(|_| handle)
}

/// fetches rows from a query
#[inline]
pub fn stmt_fetch(stmthp: *mut OCIStmt,
                  errhp: *mut OCIError,
                  nrows: u32,
                  orientation: u16,
                  offset: i32) -> Result<(), OracleError> {
    check_error(
        unsafe {
            OCIStmtFetch2(stmthp, errhp, nrows, orientation, offset, OCI_DEFAULT)
        }, Some(errhp), "oci::stmt_fetch")
}

/// allocate OCISession handle and set username, passwd attributes to it
pub fn prepare_auth(envhp: *mut OCIEnv, errhp: *mut OCIError, username: &str, passwd: &str) -> Result<*mut OCISession, OracleError> {
    let authp = handle_alloc(envhp, OCI_HTYPE_SESSION)? as *mut OCISession;

    let username_len = username.len() as u32;
    let username = CString::new(username).unwrap();

    let passwd_len = passwd.len() as u32;
    let passwd = CString::new(passwd).unwrap();

    attr_set(authp as *mut c_void, OCI_HTYPE_SESSION,
                  username.as_ptr() as *mut c_void, username_len,
                  OCI_ATTR_USERNAME, errhp)?;

    attr_set(authp as *mut c_void, OCI_HTYPE_SESSION,
                  passwd.as_ptr() as *mut c_void, passwd_len,
                  OCI_ATTR_PASSWORD, errhp)?;

    Ok(authp)
}
