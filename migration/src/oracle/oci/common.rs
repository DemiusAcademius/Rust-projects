pub use super::libc::{c_void, c_ushort, c_short, c_ulong, c_uchar, c_char, c_uint, c_int};

/// Type of handle
#[allow(dead_code)]
pub enum OCIHandleType {
    /// `OCI_HTYPE_ENV`
    Environment = 1,

    /// `OCI_HTYPE_ERROR`
    Error       = 2,

    /// `OCI_HTYPE_SVCCTX`
    Service     = 3,

    /// `OCI_HTYPE_STMT`
    Statement   = 4,

    /// `OCI_HTYPE_BIND`
    Bind        = 5,

    /// `OCI_HTYPE_DEFINE`
    Define      = 6,

    /// `OCI_HTYPE_DESCRIBE`
    Describe    = 7,

    /// `OCI_HTYPE_SERVER`
    Server      = 8,

    /// `OCI_HTYPE_SESSION`
    Session     = 9,

    /// `OCI_HTYPE_TRANS`
    Transaction = 10,
}

/// Opaque pointer to OCIEnv
#[repr(C)]
pub struct OCIEnv(c_void);

/// Opaque pointer to OCISvcCtx
#[repr(C)]
pub struct OCISvcCtx(c_void);

/// Opaque pointer to OCIServer
#[repr(C)]
pub struct OCIServer(c_void);

/// Opaque pointer to OCISession
#[repr(C)]
pub struct OCISession(c_void);

/// Opaque pointer to OCISnapshot
#[repr(C)]
pub struct OCISnapshot(c_void);

/// OCI Mode type.
/// Used in [`oci_env_nls_create`](fn.oci_env_nls_create.html),
#[allow(dead_code)]
pub enum OCIMode {
    /// `OCI_DEFAULT`. The default value, which is non-UTF-16 encoding.
    Default = 0x00000000,

    /// `OCI_THREADED`. Uses threaded environment.
    /// Internal data structures not exposed to the user are protected from concurrent
    /// accesses by multiple threads.
    Threaded = 0x00000001,

    /// `OCI_OBJECT`. Uses object features.
    Object = 0x00000002,

    /// `OCI_EVENTS`. Uses publish-subscribe notifications.
    Events = 0x00000004,

    /// `OCI_NO_UCB`. Suppresses the calling of the dynamic callback routine `OCIEnvCallback()`.
    /// The default behavior is to allow calling of `OCIEnvCallback()` when the environment
    /// is created.
    NoUcb = 0x00000040,

    /// `OCI_NO_MUTEX`. No mutual exclusion (mutex) locking occurs in this mode.
    /// All OCI calls done on the environment handle, or on handles derived from the environment
    /// handle, must be serialized.
    /// `Threaded` must also be specified when `OCI_NO_MUTEX` is specified.
    NoMutex = 0x00000080,

    /// `OCI_SUPPRESS_NLS_VALIDATION`. Suppresses NLS character validation;
    /// NLS character validation suppression is on by default beginning with
    /// Oracle Database 11g Release 1 (11.1). Use `EnableNLSValidation` to
    /// enable NLS character validation.
    SuppressNLSValidation = 0x00100000,

    /// `OCI_NCHAR_LITERAL_REPLACE_ON`. Turns on N' substitution.
    NcharLiteralReplaceOn = 0x00400000,

    /// `OCI_NCHAR_LITERAL_REPLACE_OFF`. Turns off N' substitution.
    /// If neither this mode nor `NcharLiteralReplaceOn` is used, the substitution is
    /// determined by the environment variable `ORA_NCHAR_LITERAL_REPLACE`, which can be set
    /// to `TRUE` or `FALSE`. When it is set to `TRUE`, the replacement is turned on; otherwise
    /// it is turned off, which is the default setting in OCI.
    NcharLiteralReplaceOff = 0x00800000,

    /// `OCI_ENABLE_NLS_VALIDATION`. Enables NLS character validation.
    EnableNLSValidation = 0x01000000,
}

/// Type of credentials
#[allow(dead_code)]
pub enum OCICredentialsType {
    /// `OCI_CRED_RDBMS`
    Rdbms    = 1,

    /// `OCI_CRED_EXT`
    External = 2,
}

/// Type of authentication mode
#[allow(dead_code)]
pub enum OCIAuthMode {
    /// `OCI_DEFAULT`
    Default    = 0x00000000,

    /// `OCI_MIGRATE`
    Migrate    = 0x00000001,

    /// `OCI_SYSDBA`
    Sysdba     = 0x00000002,

    /// `OCI_SYSOPER`
    Sysoper    = 0x00000004,

    /// `OCI_PRELIM_AUTH`
    PrelimAuth = 0x00000008,

    /// `OCI_STMT_CACHE`
    StmtCache  = 0x00000040,
}

/// Type of commit
#[allow(dead_code)]
pub enum OCICommitMode {
    /// `OCI_DEFAULT`
    Default    = 0x00000000,

    /// `OCI_TRANS_WRITEBATCH`
    Batch     = 0x00000001,

    /// `OCI_TRANS_WRITEIMMED`
    Immediate  = 0x00000002,

    /// `OCI_TRANS_WRITEWAIT`
    Wait       = 0x00000004,

    /// `OCI_TRANS_WRITEWAIT`
    Nowait = 0x00000008,
}


/// Type if OCI Attribute
pub enum OCIAttribute {
    /// `OCI_ATTR_SERVER`
    /// 
    /// Mode: READ/WRITE
    /// 
    /// When read, returns the pointer to the server context attribute of the service context.
    /// When changed, sets the server context attribute of the service context.
    /// Attribute Data Type: OCIServer ** / OCIServer *
    Server = 6,

    /// `OCI_ATTR_SESSION`
    /// 
    /// Mode: READ/WRITE
    /// 
    /// When read, returns the pointer to the authentication context attribute of
    /// the service context.
    /// When changed, sets the authentication context attribute of the service context.
    /// Attribute Data Type: OCISession **/ OCISession *
    Session = 7,

    /// `OCI_ATTR_PREFETCH_ROWS`
    PrefetchRows = 11,

    /// `OCI_ATTR_PREFETCH_MEMORY`
    PrefetchMemory = 13,

    /// `OCI_ATTR_USERNAME`
    /// 
    /// Mode: READ/WRITE
    /// 
    /// Specifies a user name to use for authentication.
    /// Attribute Data Type: oratext **/oratext * [oratext = c_uchar]
    Username = 22,

    /// `OCI_ATTR_PASSWORD`
    /// 
    /// Mode: WRITE
    /// 
    /// Specifies a password to use for authentication.
    /// Attribute Data Type: oratext * [oratext = c_uchar]
    Password = 23,

    /// `OCI_ATTR_CHARSET_ID`
    CharsetId = 31,


    /// `OCI_ATTR_ROWS_FETCHED`
    /// Mode: READ
    ///
    /// Specifies rows fetched in last call
    /// ttribute Data Type: ub4 *
    RowsFetched = 197,
}

/// Type of descriptor
#[allow(dead_code)]
pub enum OCIDescriptorType {
    /// `OCI_DTYPE_LOB`
    Lob = 50,

    /// `OCI_DTYPE_PARAM`
    Parameter = 53,
}

/// Type of describe attribute
#[allow(dead_code)]
pub enum OCIDescribeAttribute {
    /// `OCI_ATTR_DATA_SIZE`: maximum size of the data
    DataSize = 1,

    /// `OCI_ATTR_DATA_TYPE`: the SQL type of the column/argument
    DataType = 2,

    /// `OCI_ATTR_DISP_SIZE`: the display size
    DisplaySize = 3,

    /// `OCI_ATTR_NAME`: the name of the column/argument
    Name = 4,

    /// `OCI_ATTR_PRECISION`: precision if number type
    Precision = 5,

    /// `OCI_ATTR_SCALE`: scale if number type
    Scale = 6,

    /// `OCI_ATTR_IS_NULL`: is it null?
    IsNull = 7,

    /// `OCI_ATTR_CHAR_USED`: char length semantics
    CharUsed = 285,

    /// `OCI_ATTR_CHAR_SIZE`: char length
    CharLength = 286,
}

/// Opaque pointer to OCIStmt
#[repr(C)]
pub struct OCIStmt(c_void);

/// Opaque pointer to OCIDefine
#[repr(C)]
pub struct OCIDefine(c_void);

/// Opaque pointer to OCIBind
#[repr(C)]
pub struct OCIBind(c_void);

/// Opaque pointer to OCILobLocator
#[repr(C)]
pub struct OCILobLocator(c_void);


/// Type of OCIStmtPrepare2 mode
pub enum OCIStmtPrepare2Mode {
    /// `OCI_DEFAULT`
    Default = 0x00000000,
}

/// Type of syntax
pub enum OCISyntax {
    /// `OCI_NTV_SYNTAX`
    NtvSyntax = 1,
}


/// Oracle datatype
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum OCIDataType {
    /// Unsupported data type
    Unsupported = 0,

    /// `SQLT_CHR`: (ORANET TYPE) character string
    Char = 1,

    /// DATE !!! `SQLT_DATE`: ANSI Date
    Date = 12,

    /// TIMESTAMP `SQLT_TIMESTAMP`: `TIMESTAMP`
    Timestamp = 180,

    /// `SQLT_TIMESTAMP_TZ`: `TIMESTAMP WITH TIME ZONE`
    TimestampWithTz  = 188,

    /// `SQLT_TIMESTAMP_LTZ`: `TIMESTAMP WITH LOCAL TZ`
    TimestampWithLocalTz = 232,

    /// `SQLT_INTERVAL_YM`: `INTERVAL YEAR TO MONTH`
    IntervalYearToMonth = 189,

    /// `SQLT_INTERVAL_DS`: `INTERVAL DAY TO SECOND`
    IntervalDayToSecond = 190,

    /// `SQLT_CLOB`: character lob
    Clob = 112,

    /// `SQLT_BLOB`: binary lob
    Blob = 113,

    /// `SQLT_INT`: (ORANET TYPE) integer
    Int = 3,

    /// `SQLT_UIN`: unsigned integer
    Uint = 68,

    /// `SQLT_FLT`: (ORANET TYPE) Floating point number
    Float = 4,

    /// `SQLT_PDN`: (ORANET TYPE) Packed Decimal Numeric
    PackedDecimalNumber = 7,

    /// `SQLT_LNG: Long character
    Long = 8,

    /// `SQLT_BIN`: binary data (DTYBIN)
    Binary = 23,

    /// `SQLT_NUM`: (ORANET TYPE) oracle numeric
    Numeric = 2,

    /// `SQLT_NTY`: named object type
    NamedObject = 108,

    /// `SQLT_REF`: ref type
    Ref = 110,

    /// `SQLT_VST`: OCIString type
    OCIString = 155,

    /// `SQLT_VNU`: NUM with preceding length byte
    NumericWithLength = 6,
}

/// Orientation
#[allow(dead_code)]
pub enum OCIOrientation {
    
    /// `OCI_FETCH_CURRENT`
    FetchCurrent = 0x00000001,

    /// `OCI_FETCH_NEXT`
    FetchNext = 0x00000002,

    /// `OCI_FETCH_FIRST`
    FetchFirst = 0x00000004,

    /// `OCI_FETCH_LAST`
    FetchLast = 0x00000008,

    /// `OCI_FETCH_PRIOR`
    FetchPrior = 0x00000010,

    /// `OCI_FETCH_ABSOLUTE`
    FetchAbsolute = 0x00000020,

    /// `OCI_FETCH_RELATIVE`
    FetchRelative = 0x00000040,
}

/// Type of temp blob
#[allow(dead_code)]
pub enum OCITempLobType {
    ///  Default
    Default = 0,

    /// `OCI_TEMP_BLOB`
    Blob = 1,

    /// `OCI_TEMP_CLOB`
    Clob = 2,
}

/// Flags of BLOB Read/Write
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OCILobPiece {

    /// `OCI_ONE_PIECE`
    OnePiece = 0,

    /// `OCI_FIRST_PIECE`
    FirstPiece = 1,

    /// `OCI_NEXT_PIECE`
    NextPiece = 2,

    /// `OCI_LAST_PIECE`
    LastPiece = 3
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OCICharset {
    Default = 0,
    WE8ISO8859P1 = 31,
    EE8ISO8859P2 = 32
}