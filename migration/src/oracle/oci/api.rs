#![allow(dead_code)]

use std;
use std::ffi::CStr;
use std::ffi::CString;
use std::mem;
use std::ptr;

use super::common::*;
use super::errors::*;
use super::ffi::*;

/// Binds [`OCIEnvNlsCreate()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci16rel001.htm#LNOCI17114).
pub fn oci_env_nls_create(mode: OCIMode, charset: u16) -> Result<*mut OCIEnv, OracleError> {
    let mut handle = ptr::null_mut();

    // let charset: c_ushort = 1000; // UTF 16 charset

    let res = unsafe {
        OCIEnvNlsCreate(&mut handle, // envp
                        mode as c_uint, // mode
                        ptr::null_mut(), // ctxp
                        None, // malocfp
                        None, // ralocfp
                        None, // mfreefp
                        0, // xtramem_sz
                        ptr::null_mut(), // usrmempp
                        charset, // charset
                        0 /* ncharset */)
    };
    match check_error(res, None, "ffi::oci_env_nls_create") {
        None => {
            Ok(handle)
        }
        Some(err) => Err(err),
    }
}

pub fn oci_nsl_charset_name_to_id(envh: *mut OCIEnv, charset_name: String) -> u16 {
    let charset_name = CString::new(charset_name).unwrap();
    unsafe { OCINlsCharSetNameToId(envh, charset_name.as_ptr() as *const c_uchar) }
}

pub fn oci_nsl_charset_id_to_name(envh: &mut OCIEnv, charset_id: u16) -> Result<String, OracleError> {
    let mut buffer: [c_uchar; 256] = [0; 256];
    let buf_ptr = buffer.as_ptr();
    let res = unsafe { OCINlsCharSetIdToName(envh, buf_ptr, 256, charset_id) };

    match check_error(res, None, "ffi::oci_nsl_charset_id_to_name") {
        None => {
            let cstring = unsafe { CString::from_raw(buf_ptr as *mut c_char) };
            Ok(cstring.into_string().unwrap())
        }
        Some(err) => Err(err),
    }    
}

/// Binds [`OCIHandleAlloc()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci16rel002.htm#LNOCI17134).
pub fn oci_handle_alloc(envh: *mut OCIEnv,
                        htype: OCIHandleType)
                        -> Result<*mut c_void, OracleError> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
        OCIHandleAlloc(envh as *const _, // parenth
                       &mut handle, // hndlpp
                       htype as c_uint, // type
                       0, // xtramem_sz
                       ptr::null_mut() /* usrmempp */)
    };
    match check_error(res, None, "ffi::oci_handle_alloc") {
        None => Ok(handle),
        Some(err) => Err(err),
    }
}

/// Binds [`OCIServerAttach()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci16rel001.htm#LNOCI17119).
pub fn oci_server_attach(server_handle: *mut OCIServer,
                         error_handle: *mut OCIError,
                         db: String,
                         mode: OCIMode)
                         -> Result<(), OracleError> {
    let db_len = db.len();
    let db = CString::new(db).unwrap();
                             
    let res = unsafe {
        OCIServerAttach(server_handle, // srvhp
                        error_handle, // errhp
                        db.as_ptr() as *const c_uchar, // dblink
                        db_len as c_int, // dblink_len
                        mode as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_server_attach") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

/// Binds [`OCIAttrSet()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci16rel002.htm#LNOCI17131).
pub fn oci_attr_set(handle: *mut c_void,
                    htype:  OCIHandleType,
                    value:  *mut c_void,    
                    size:   c_uint,                
                    attr_type:    OCIAttribute,
                    error_handle: *mut OCIError)
                    -> Result<(), OracleError> {
                        /*
    let size: c_uint = match attr_type {
        OCIAttribute::Username | OCIAttribute::Password => {
            println!("attr: {:?}", unsafe { CStr::from_ptr(value as *const c_char).to_bytes() } );        
            unsafe { CStr::from_ptr(value as *const c_char).to_bytes().len() as c_uint }
        }
        _ => 0,
    };
    */

    let res = unsafe {
        OCIAttrSet(handle, // trgthndlp
                   htype as c_uint, // trghndltyp
                   value, // attributep
                   size, // size
                   attr_type as c_uint, // attrtype
                   error_handle /* errhp */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_attr_set") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

/// Binds [`OCIAttrGet()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci16rel002.htm#LNOCI17130).
pub fn oci_attr_get(attr_handle: *mut c_void,
                    htype: OCIHandleType,
                    value: *mut c_void,
                    attr_type: OCIAttribute,
                    error_handle: *mut OCIError)
                    -> Result<isize, OracleError> {
    let mut attribute_size = 0;
    let res = unsafe {
        OCIAttrGet(attr_handle as *const _, // trgthndlp
                   htype as c_uint, // trghndltyp
                   value, // attributep
                   &mut attribute_size, // sizep
                   attr_type as c_uint, // attrtype
                   error_handle /* errhp */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_attr_get") {
        None => Ok(attribute_size as isize),
        Some(err) => Err(err),
    }
}

/// Binds [`OCISessionBegin()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci16rel001.htm#LNOCI17121).
pub fn oci_session_begin(service_handle: *mut OCISvcCtx,
                         error_handle: *mut OCIError,
                         session_handle: *mut OCISession,
                         credentials_type: OCICredentialsType,
                         mode: OCIAuthMode)
                         -> Result<(), OracleError> {
    let res = unsafe {
        OCISessionBegin(service_handle, // svchp
                        error_handle, // errhp
                        session_handle, // usrhp
                        credentials_type as c_uint, // credt
                        mode as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_session_begin") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

/// Binds [`OCISessionEnd()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci16rel001.htm#LNOCI17122).
pub fn oci_session_end(service_handle: *mut OCISvcCtx,
                       error_handle: *mut OCIError,
                       session_handle: *mut OCISession)
                       -> Result<(), OracleError> {
    let res = unsafe {
        OCISessionEnd(service_handle, // svchp
                      error_handle, // errhp
                      session_handle, // usrhp
                      OCIAuthMode::Default as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_session_end") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

/// Binds [`OCIServerDetach()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci16rel001.htm#LNOCI17120).
pub fn oci_server_detach(server_handle: *mut OCIServer,
                         error_handle: *mut OCIError)
                         -> Result<(), OracleError> {
    let res = unsafe { OCIServerDetach(server_handle, error_handle, OCIMode::Default as c_uint) };
    match check_error(res, Some(error_handle), "ffi::oci_server_detach") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

/// Binds [`OCIHandleFree()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci16rel002.htm#LNOCI17135).
pub fn oci_handle_free(handle: *mut c_void, htype: OCIHandleType) -> Result<(), OracleError> {
    let res = unsafe { OCIHandleFree(handle, htype as c_uint) };
    match check_error(res, None, "ffi::oci_handle_free") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

/// Binds [`OCIStmtPrepare2()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci17msc001.htm#LNOCI17168).
pub fn oci_stmt_prepare2(service_handle: *mut OCISvcCtx,
                         error_handle: *mut OCIError,
                         stmt_text: &String,
                         stmt_hash: &String)
                         -> Result<*mut OCIStmt, OracleError> {
    let mut stmt_handle = ptr::null_mut();
    let res = unsafe {
        OCIStmtPrepare2(service_handle, // svchp
                        &mut stmt_handle, // stmtp
                        error_handle, // errhp
                        stmt_text.as_ptr(), // stmttext
                        stmt_text.len() as c_uint, // stmt_len
                        stmt_hash.as_ptr(), // key
                        stmt_hash.len() as c_uint, // key_len
                        OCISyntax::NtvSyntax as c_uint, // language
                        OCIStmtPrepare2Mode::Default as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_stmt_prepare2") {
        None => Ok(stmt_handle),
        Some(err) => Err(err),
    }
}

/// Binds [`OCIStmtExecute()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci17msc001.htm#LNOCI17163).
pub fn oci_stmt_execute(service_handle: *mut OCISvcCtx,
                        stmt_handle: *mut OCIStmt,
                        error_handle: *mut OCIError,
                        iterations: u32)
                        -> Result<(), OracleError> {
    let res = unsafe {
        OCIStmtExecute(service_handle, // svchp
                       stmt_handle, // stmtp
                       error_handle, // errhp
                       iterations, // iters
                       0 as c_uint, // rowoff
                       ptr::null(), // snap_in
                       ptr::null_mut(), // snap_out
                       OCIMode::Default as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_stmt_execute") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

pub fn oci_trans_commit(service_handle: *mut OCISvcCtx, error_handle: *mut OCIError, mode: OCICommitMode) -> Result<(), OracleError> {
    let res = unsafe {
        OCITransCommit(service_handle, // svchp
                       error_handle, // errhp
                       mode as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_trans_commit") {
        None => Ok(()),
        Some(err) => Err(err),
    }        
}

pub fn oci_stmt_define_by_pos(stmt_handle: *mut OCIStmt,
                              error_handle: *mut OCIError,
                              position: usize,
                              valuep: *const u8,
                              indp: *const c_short,
                              size: isize,
                              rlenp: *const c_ushort,
                              data_type: OCIDataType)
                              -> Result<(), OracleError> {

    let mut handle = ptr::null_mut();

    let res = unsafe {
        OCIDefineByPos(stmt_handle, // stmtp
                       &mut handle, // defnpp
                       error_handle, // errhp
                       position as c_uint, // position
                       valuep, // valuep
                       size as c_int, // value_sz
                       data_type as c_ushort, // dty
                       indp, // indp
                       rlenp, // rlenp
                       ptr::null(), // rcodep
                       OCIMode::Default as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_stmt_define") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

pub fn oci_stmt_bind_by_name(stmt_handle: *mut OCIStmt,
                             error_handle: *mut OCIError,
                             placeholder: &String,
                             valuep: *const u8,
                             indp: *const c_short,
                             size: usize,
                             alenp: *const u16,
                             data_type: OCIDataType)
                             -> Result<(), OracleError> {

    let mut handle = ptr::null_mut();

    let res = unsafe {
        OCIBindByName(stmt_handle, // stmtp
                      &mut handle, // bindpp
                      error_handle, // errhp
                      placeholder.as_ptr(), // placeholder
                      placeholder.len() as c_uint, // placeh_len
                      valuep, // valuep
                      size as c_int, // value_sz
                      data_type as c_ushort, // dty
                      indp, // indp
                      alenp, // ptr::null_mut(), // alenp
                      ptr::null(), // rcodep
                      1, // maxarr_len
                      ptr::null(), // curelep
                      OCIMode::Default as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_stmt_define") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

pub fn oci_stmt_bind_by_pos(stmt_handle: *mut OCIStmt,
                            error_handle: *mut OCIError,
                            position: usize,
                            valuep: *const u8,
                            indp: *const c_short,
                            size: isize,
                            alenp: *const u16,
                            data_type: OCIDataType)
                             -> Result<*mut OCIBind, OracleError> {

    let mut handle = ptr::null_mut();

    let res = unsafe {
        OCIBindByPos(stmt_handle, // stmtp
                     &mut handle, // bindpp
                     error_handle, // errhp
                     position as c_uint, // position
                     valuep, // valuep
                     size as c_int, // value_sz
                     data_type as c_ushort, // dty
                     indp, // indp
                     alenp, // ptr::null_mut(), // alenp
                     ptr::null(), // rcodep
                     1, // maxarr_len
                     ptr::null(), // curelep
                     OCIMode::Default as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_stmt_define") {
        None => Ok(handle),
        Some(err) => Err(err),
    }
}

pub fn oci_stmt_fetch(stmt_handle: *mut OCIStmt,
                      error_handle: *mut OCIError,
                      nrows: u32,
                      orientation: OCIOrientation,
                      fetch_offset: i32)
                      -> Result<(), OracleError> {
    let res = unsafe {
        OCIStmtFetch2(stmt_handle, // stmtp
                      error_handle, // errhp
                      nrows, // nrows
                      orientation as c_ushort, // orientation
                      fetch_offset, // fetchOffset
                      OCIMode::Default as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_stmt_fetch") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

/// Binds [`OCIStmtRelease()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci17msc001.htm#LNOCI17169).
pub fn oci_stmt_release(stmt_handle: *mut OCIStmt,
                        error_handle: *mut OCIError,
                        stmt_hash: &String)
                        -> Result<(), OracleError> {
    let res = unsafe {
        OCIStmtRelease(stmt_handle, // stmtp
                       error_handle, // errhp
                       stmt_hash.as_ptr(), // key
                       stmt_hash.len() as c_uint, // keylen
                       OCIMode::Default as c_uint /* mode */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_stmt_release") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

/// Binds [`OCIParamGet()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci16rel002.htm#LNOCI17136).
pub fn oci_param_get(stmt_handle: *mut OCIStmt,
                     error_handle: *mut OCIError,
                     position: usize)
                     -> Result<*mut c_void, OracleError> {
    let mut parameter_descriptor = ptr::null_mut();
    let res = unsafe {
        OCIParamGet(stmt_handle as *const _, // hndlp
                    OCIHandleType::Statement as c_uint, // htype
                    error_handle, // errhp
                    &mut parameter_descriptor, // parmdpp
                    position as c_uint /* pos */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_param_get") {
        None => Ok(parameter_descriptor),
        Some(err) => Err(err),
    }
}

pub fn oci_descriptor_alloc(envh: *mut OCIEnv,
                            desctype: OCIDescriptorType) -> Result<*mut c_void, OracleError> {
    let mut handle = ptr::null_mut();
    let res = unsafe {
        OCIDescriptorAlloc(envh as *const _, // parenth
                       &mut handle, // descpp
                       desctype as c_uint, // type
                       0, // xtramem_sz
                       ptr::null_mut() /* usrmempp */)
    };
    match check_error(res, None, "ffi::oci_descriptor_alloc") {
        None => Ok(handle),
        Some(err) => Err(err),
    }
}

pub fn oci_descriptor_free(handle: *mut c_void, desctype: OCIDescriptorType) -> Result<(), OracleError> {
    let res = unsafe { OCIDescriptorFree(handle, desctype as c_uint) };
    match check_error(res, None, "ffi::oci_descriptor_free") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}


pub fn oci_lob_create_temporary(service_handle: *mut OCISvcCtx,
                                error_handle: *mut OCIError,
                                locp: *mut OCILobLocator,
                                lobtype: OCITempLobType) -> Result<(), OracleError> {
    let res = unsafe {
        OCILobCreateTemporary(service_handle, // svchp
                    error_handle, // errhp
                    locp, // locp
                    0, // csid OCI_DEFAULT
                    0, // csfrm OCI_DEFAULT
                    lobtype as c_uchar,
                    false, // cache
                    10, /* duration OCI_DURATION_SESSION */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_lob_create_temporary") {
        None => Ok(()),
        Some(err) => Err(err),
    }
} 

pub fn oci_lob_free_temporary(service_handle: *mut OCISvcCtx,
                              error_handle: *mut OCIError,
                              locp: *mut OCILobLocator) -> Result<(), OracleError> {
    let res = unsafe {
        OCILobFreeTemporary(service_handle, // svchp
                    error_handle, // errhp
                    locp, /* locp */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_lob_free_temporary") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

pub fn oci_lob_get_length(service_handle: *mut OCISvcCtx,
                          error_handle: *mut OCIError,
                          locator: *mut OCILobLocator) -> Result<u32, OracleError> {                              
    let mut length: u32 = 0;
    let length_ptr = unsafe { mem::transmute::<&mut u32, *const c_uint>(&mut length) };
                              
    let res = unsafe {
        OCILobGetLength(service_handle, // svchp
                         error_handle, // errhp
                         locator, // locp
                         length_ptr /* lenp */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_lob_get_length") {
        None => Ok(length),
        Some(err) => Err(err),
    }
}

/// return Ok(finita, rows) sau Err(error), finita -> true, end of read, false -> need data 
pub fn oci_lob_read(service_handle: *mut OCISvcCtx,
                    error_handle: *mut OCIError,
                    locator: *mut OCILobLocator,
                    offset: u32,
                    buffer: *const u8, // buffer
                    buffer_len: u32,       // buffer len
                    ) -> Result<(bool, u32), OracleError> {
    let mut amt: u32 = std::u32::MAX;
    let amtp = unsafe { mem::transmute::<&mut u32, *const c_uint>(&mut amt) };
                        
    let res = unsafe {
        OCILobRead(service_handle, // svchp
                   error_handle, // errhp
                   locator, // locp
                   amtp,
                   offset,
                   buffer,
                   buffer_len,
                   ptr::null_mut(), // ctxp
                   None, // cbfp
                   0, // csid
                   0, /* csfrm SQLCS_IMPLICIT(may be 1) OCI_DEFAULT(0) */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_lob_read") {
        None => Ok((true,amt)),
        Some(err) => 
            if err.code == 99 { Ok((false, amt)) } else { Err(err) }
    }
}

pub fn oci_lob_write(service_handle: *mut OCISvcCtx,
                    error_handle: *mut OCIError,
                    locator: *mut OCILobLocator,
                    write_len: u32,
                    offset: u32,
                    buffer: *const u8, // buffer
                    buffer_len: u32,   // buffer len
                    piece: OCILobPiece                    
                    ) -> Result<(), OracleError> {
    let mut amt = write_len;
    let amtp = unsafe { mem::transmute::<&mut u32, *const c_uint>(&mut amt) };
                        
    let res = unsafe {
        OCILobWrite(service_handle, // svchp
                    error_handle, // errhp
                    locator, // locp
                    amtp,
                    offset,
                    buffer,
                    buffer_len,
                    piece as u8,
                    ptr::null_mut(), // ctxp
                    None, // cbfp
                    0, // csid
                    0, /* csfrm SQLCS_IMPLICIT(may be 1) OCI_DEFAULT(0) */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_lob_write") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}

pub fn oci_lob_trim(service_handle: *mut OCISvcCtx,
                   error_handle: *mut OCIError,
                   locator: *mut OCILobLocator,
                   new_len: u32) -> Result<(), OracleError> {
    let res = unsafe {
        OCILobTrim(service_handle, // svchp
                    error_handle, // errhp
                    locator, // locp
                    new_len /* new_len */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_lob_trim") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}


pub fn oci_lob_copy(service_handle: *mut OCISvcCtx,
                    error_handle: *mut OCIError,
                    dst_locp: *mut OCILobLocator,
                    src_locp: *mut OCILobLocator) -> Result<(), OracleError> {

    println!("u64 max: {}", std::u64::MAX);

    let res = unsafe {
        OCILobCopy2(service_handle, // svchp
                    error_handle, // errhp
                    dst_locp, // dst_locp
                    src_locp, // src_locp,
                    std::u64::MAX, // amout
                    0, // dst_offset
                    0 /* src_offset */)
    };
    match check_error(res, Some(error_handle), "ffi::oci_lob_copy") {
        None => Ok(()),
        Some(err) => Err(err),
    }
}