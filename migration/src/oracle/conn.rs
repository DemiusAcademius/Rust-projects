use std::ffi::CString;

use super::oci;

use super::env;
use super::stmt;
use super::typed_query;
use super::bindings;

pub struct Connection {
    pub env:            env::Environment,
    pub service_handle: *mut oci::OCISvcCtx,
    server_handle:      *mut oci::OCIServer,
    session_handle:     *mut oci::OCISession,
}

impl Connection {
    pub fn new(username: String,
               password: String,
               database: String,
               charset: oci::OCICharset) -> Result<Connection, oci::OracleError> {
                   /*
        let charset_id = match charset {
            oci::OCICharset::Default => 0,
            // oci::OCICharset::WE8ISO8859P1 => 31,
            // oci::OCICharset::EE8ISO8859P2 => 32
            oci::OCICharset::WE8ISO8859P1 => 1,
            oci::OCICharset::EE8ISO8859P2 => 2
        };
        */
        // Initialize environment
        let env = env::Environment::new(0)?;

        /*
        if charset == oci::OCICharset::EE8ISO8859P2 {
            let charset_id = oci::oci_nsl_charset_name_to_id(env.handle, "EE8ISO8859P2".to_string());
            println!("id of EE8ISO8859P2 is: {}", charset_id);
        } else if charset == oci::OCICharset::WE8ISO8859P1 {
            let charset_id = oci::oci_nsl_charset_name_to_id(env.handle, "WE8ISO8859P1".to_string());
            println!("id of WE8ISO8859P1 is: {}", charset_id);
        }
        */

        // Allocate the server handle
        let server_handle =
            oci::oci_handle_alloc(env.handle,
                 oci::OCIHandleType::Server)? as *mut oci::OCIServer;

        // Allocate the service context handle
        let service_handle =
            oci::oci_handle_alloc(env.handle,
                 oci::OCIHandleType::Service)? as *mut oci::OCISvcCtx;

        // Allocate the session handle
        let session_handle =
            oci::oci_handle_alloc(env.handle,
                 oci::OCIHandleType::Session)? as *mut oci::OCISession;

        oci::oci_server_attach(server_handle, env.error_handle, database, oci::OCIMode::Default)?;

        // Set attribute server context in the service context
        oci::oci_attr_set(service_handle as *mut oci::c_void,
                                       oci::OCIHandleType::Service,
                                       server_handle as *mut oci::c_void,
                                       0,
                                       oci::OCIAttribute::Server,
                                       env.error_handle)?;
            
        // Set attribute username in the session context
        let username_len = username.len();
        let username = CString::new(username).unwrap();
        oci::oci_attr_set(session_handle as *mut oci::c_void,
                                        oci::OCIHandleType::Session,
                                        username.as_ptr() as *mut oci::c_void,
                                        username_len as u32,
                                        oci::OCIAttribute::Username, env.error_handle)?;

        // Set attribute password in the session context
        let password_len = password.len();
        let password = CString::new(password).unwrap();
        oci::oci_attr_set(session_handle as *mut oci::c_void,
                                       oci::OCIHandleType::Session,
                                       password.as_ptr() as *mut oci::c_void,
                                       password_len as u32,
                                       oci::OCIAttribute::Password, env.error_handle)?;

        // Begin session
        oci::oci_session_begin(service_handle, env.error_handle, session_handle,
                                            oci::OCICredentialsType::Rdbms,
                                            oci::OCIAuthMode::Default)?;

        // Set session context in the service context
        oci::oci_attr_set(service_handle as *mut oci::c_void,
                                       oci::OCIHandleType::Service,
                                       session_handle as *mut oci::c_void,
                                       0,
                                       oci::OCIAttribute::Session,
                                       env.error_handle)?;
        Ok(
            Connection {
                env:            env,
                service_handle: service_handle,
                server_handle:  server_handle,
                session_handle: session_handle,
            }
        )
    }

    pub fn commit(&self, mode: oci::OCICommitMode) -> Result<(), oci::OracleError> {
        oci::oci_trans_commit(self.service_handle, self.env.error_handle, mode)
    }

    pub fn prepare_statement<S: Into<String>>(&self, stmt_text: S, bindmap: Option<bindings::Bindmap>) -> Result<stmt::Statement, oci::OracleError> {
        stmt::Statement::new(self, stmt_text.into(), bindmap)
    }

    pub fn query<S: Into<String>>(&self, stmt_text: S) -> typed_query::QueryBuilder {
        typed_query::QueryBuilder::new(self, stmt_text.into())
    }

    pub fn execute<S: Into<String>>(&self, stmt_text: S) -> Result<(), oci::OracleError> {
        let statement = stmt::Statement::new(self, stmt_text.into(), None)?;
        statement.execute(1)
    }    
}

impl Drop for Connection {
    fn drop(&mut self) {
        oci::oci_session_end(self.service_handle, self.env.error_handle, self.session_handle)
            .ok().expect("oci_session_end failed");
        oci::oci_server_detach(self.server_handle, self.env.error_handle)
            .ok().expect("oci_server_detach failed");
        oci::oci_handle_free(self.session_handle as *mut oci::c_void,
                                     oci::OCIHandleType::Session)
            .ok().expect("oci_handle_free (session_handle) failed");
        oci::oci_handle_free(self.service_handle as *mut oci::c_void,
                                     oci::OCIHandleType::Service)
            .ok().expect("oci_handle_free (service_handle) failed");
        oci::oci_handle_free(self.server_handle as *mut oci::c_void,
                                     oci::OCIHandleType::Server)
            .ok().expect("oci_handle_free (server_handle) failed");
    }
}