use std::error;
use std::fmt;
use std::ptr;

use super::common::*;

/// Opaque pointer to OCIError
#[repr(C)]
pub struct OCIError(c_void);

/// Represent Oracle error.
#[derive(Debug)]
pub struct OracleError {
    /// Oracle error code.
    pub code:     isize,
    /// Message.
    message:  String,
    /// Function where the error occurred.
    location: String,
}

impl fmt::Display for OracleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!{f, "\n\n  Error code: {}\n  Error message: {}\n  Where: {}\n\n",
               self.code, self.message, self.location}
    }
}

impl error::Error for OracleError {
    fn description(&self) -> &str {
        "Oracle error"
    }
}

#[link(name = "clntsh")]
extern "C" {
    fn OCIErrorGet(hndlp: *mut c_void, recordno: c_uint, sqlstate: *mut c_uchar,
                   errcodep: *mut c_int, bufp: *mut c_uchar, bufsiz: c_uint, _type: c_uint) -> c_int;

}

/// Binds [`OCIErrorGet()`](http://docs.oracle.com/cd/E11882_01/appdev.112/e10646/oci17msc007.htm#LNOCI17287).
pub fn oci_error_get(error_handle: *mut OCIError, location: &str) -> OracleError {
    let errc: *mut isize = &mut 0;
    let buf = String::with_capacity(3072);
    unsafe {
        OCIErrorGet(
            error_handle as *mut c_void,                                  // hndlp
            1,                                                            // recordno
            ptr::null_mut(),                                              // sqlstate
            errc as *mut c_int,                                           // errcodep
            buf.as_ptr() as *mut c_uchar,                                 // bufp
            buf.capacity() as c_uint,                                     // bufsiz
            OCIHandleType::Error as c_uint                                // type
        )
    };
    OracleError {code: unsafe { *errc }, message: buf, location: location.to_string()}
}

/// Convert oracle error codes to [`OracleError`](struct.OracleError.html).
pub fn check_error(code: c_int,
                   error_handle: Option<*mut OCIError>,
                   location: &str) -> Option<OracleError> {
    let by_handle = match error_handle {
        Some(handle) => {
            let mut error = oci_error_get(handle, location);
            if error.code == 24347 {
                error.message = "NULL column in a aggregate function".to_string()                
            }
            Some(error)
        }
        None         => None,
    };
    match code {
        0     => None,
        100   => Some(OracleError {
            code: code as isize, message: "No data".to_string(), location: location.to_string()
        }),
        -2    => Some(OracleError {
            code: code as isize, message: "Invalid handle".to_string(), location: location.to_string()
        }),
        99    => Some(OracleError {
            code: code as isize, message: "Need data".to_string(), location: location.to_string()
        }),
        -3123 => Some(OracleError {
            code: code as isize, message: "Still executing".to_string(),
            location: location.to_string()
        }),
        -1    => Some(by_handle.unwrap_or(OracleError {
            code: code as isize, message: "Error with no details".to_string(),
            location: location.to_string()
        })),
        1     => Some(by_handle.unwrap_or(OracleError {
            code: code as isize, message: "Success with info".to_string(),
            location: location.to_string()
        })),
        _     => panic!("Unknown return code"),
    }
}