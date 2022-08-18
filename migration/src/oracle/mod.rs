mod oci;

pub use oci::OracleError;

/// Oracle environment
pub struct Environment {
    envhp: *mut oci::OCIEnv,
    errhp: *mut oci::OCIError
}

/// Connection to Oracle and server context
pub struct Connection {
    errhp: *mut oci::OCIError,
    srvhp: *mut oci::OCIServer,
    svchp: *mut oci::OCISvcCtx,
    authp: *mut oci::OCISession
}

/// Prepared statement
pub struct Statement {
    errhp:  *mut oci::OCIError,
    svchp:  *mut oci::OCISvcCtx,
    stmthp: *mut oci::OCIStmt
}

impl Environment {

    /// Create new environment; start here
    pub fn new() -> Result<Environment, OracleError> {
        let envhp = oci::env_create()?;
        let errhp = oci::handle_alloc(envhp, oci::OCI_HTYPE_ERROR)? as *mut oci::OCIError;
        Ok(Environment{ envhp, errhp })
    }

    /// Connect to database
    pub fn connect(&mut self, db: &str, username: &str, passwd: &str) -> Result<Connection, OracleError> {
        let srvhp = oci::handle_alloc(self.envhp, oci::OCI_HTYPE_SERVER)? as *mut oci::OCIServer;
        let svchp = oci::handle_alloc(self.envhp, oci::OCI_HTYPE_SVCCTX)? as *mut oci::OCISvcCtx;

        let res = oci::server_attach(srvhp, self.errhp, db);
        if let Err(err) = res {
            free_server_handlers(srvhp, svchp);
            return Err(err);
        };

        // set attribute server context in the service context
        oci::attr_set(svchp as *mut oci::c_void, 
                      oci::OCI_HTYPE_SVCCTX, 
                      srvhp as *mut oci::c_void, 
                      0, 
                      oci::OCI_ATTR_SERVER, 
                      self.errhp)?;

        let authp = oci::prepare_auth(self.envhp, self.errhp, username, passwd)?;

        let res = oci::session_begin(svchp, self.errhp, authp);
        if let Err(err) = res {
            free_session_handler(authp);
            free_server_handlers(srvhp, svchp);
            return Err(err);
        };

        // set session context in the service context
        oci::attr_set(svchp as *mut oci::c_void, oci::OCI_HTYPE_SVCCTX, 
            authp as *mut oci::c_void, 0,
            oci::OCI_ATTR_SESSION, self.errhp)?;
    
        return Ok( Connection::new(self.errhp, srvhp, svchp, authp) );
    }
}

impl Drop for Environment {    
    fn drop(&mut self) { 
        oci::handle_free(self.errhp as *mut oci::c_void, oci::OCI_HTYPE_ERROR);
        oci::handle_free(self.envhp as *mut oci::c_void, oci::OCI_HTYPE_ENV);
        oci::terminate();
    }
}

impl Connection {
    fn new(errhp: *mut oci::OCIError,
        srvhp: *mut oci::OCIServer,
        svchp: *mut oci::OCISvcCtx,
        authp: *mut oci::OCISession) -> Connection {
        Connection { errhp, srvhp, svchp, authp }
    }

    /// commit transaction with NO-WAIT option
    pub fn commit(&mut self) -> Result<(), OracleError> {
        oci::commit(self.svchp, self.errhp)
    }

    /// rollback transation
    pub fn rollback(&mut self) -> Result<(), OracleError> {
        oci::rollback(self.svchp, self.errhp)
    }

    /// Prepare oracle statement
    pub fn prepare(&mut self, sql: &str) -> Result<Statement, OracleError> {
        let stmthp = oci::stmt_prepare(self.svchp, self.errhp, sql)?;
        Ok( Statement::new(self.errhp, self.svchp, stmthp) )
    }
}

impl Drop for Connection {
    fn drop(&mut self) { 
        oci::session_end(self.svchp, self.errhp, self.authp);
        oci::server_detach(self.srvhp, self.errhp);
        free_session_handler(self.authp);
        free_server_handlers(self.srvhp, self.svchp);
    }
}

impl Statement {
    fn new(errhp:  *mut oci::OCIError,
           svchp:  *mut oci::OCISvcCtx,
           stmthp: *mut oci::OCIStmt) -> Statement {
        Statement { errhp, svchp, stmthp }
    }
}

impl Drop for Statement {
    fn drop(&mut self) { 
        oci::stmt_release(self.stmthp, self.errhp);
    }
}

fn free_session_handler(authp: *mut oci::OCISession) {
    if !authp.is_null() {
        oci::handle_free(authp as *mut oci::c_void, oci::OCI_HTYPE_SESSION);
    }
}

fn free_server_handlers(srvhp: *mut oci::OCIServer, svchp: *mut oci::OCISvcCtx) {
    if !svchp.is_null() {
        oci::handle_free(svchp as *mut oci::c_void, oci::OCI_HTYPE_SVCCTX);
    }
    if !srvhp.is_null() {
        oci::handle_free(srvhp as *mut oci::c_void, oci::OCI_HTYPE_SERVER);
    }
}

use num_traits::int::PrimInt;

/// incapsulate Oracle SQL Types
#[derive(Debug)]
struct SqlType<T> {
    dtype: u16,
    size:  u32,
    align: u32
}

impl 

const u32_sqltype: SqlType = SqlType { dtype: oci::SQLT_INT, size: 4, align: 4 };

constexpr static const SqlType int_type(std::int32_t value_sz) noexcept
{
return SqlType{SQLT_INT, value_sz};
}

constexpr static const SqlType double_type() noexcept
{
return SqlType{SQLT_FLT, sizeof(double)};
}

constexpr static const SqlType date_type() noexcept
{
return SqlType{SQLT_DAT, 7};
}

constexpr static const SqlType string_type() noexcept
{
return SqlType{SQLT_CHR, 1024};
}

static const boost::gregorian::date date_from_sql(const char* buffer_ptr) noexcept;
static void date_to_sql(char* buffer_ptr, boost::gregorian::date date) noexcept;
