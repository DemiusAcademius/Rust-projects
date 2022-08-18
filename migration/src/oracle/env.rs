use super::oci;

pub struct Environment {
    pub handle:       *mut oci::OCIEnv,
    pub error_handle: *mut oci::OCIError,
}

impl Environment {
    pub fn new(charset_id: u16) -> Result<Environment, oci::OracleError> {
        let handle = oci::oci_env_nls_create(oci::OCIMode::Default, charset_id)?;
        let error_handle = oci::oci_handle_alloc(handle, oci::OCIHandleType::Error)? as *mut oci::OCIError;
        Ok(Environment {handle: handle, error_handle: error_handle})
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        oci::oci_handle_free(self.error_handle as *mut oci::c_void, oci::OCIHandleType::Error)
            .ok().expect("oci_handle_free (error_handle) failed");
        oci::oci_handle_free(self.handle as *mut oci::c_void, oci::OCIHandleType::Environment)
            .ok().expect("oci_handle_free (environment handle) failed");
    }
}