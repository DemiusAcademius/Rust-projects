#![allow(dead_code)]

use super::common::*;
use super::errors::*;

#[link(name = "clntsh")]
extern "C" {
    pub fn OCINlsCharSetNameToId(usrhp: *mut OCIEnv, name: *const c_uchar) -> c_ushort;
    pub fn OCINlsCharSetIdToName(usrhp: *mut OCIEnv, name: *const c_uchar, name_len: c_uint, charset_id: c_ushort) -> c_int;

    pub fn OCIEnvNlsCreate(envp: *mut *mut OCIEnv,
                           mode: c_uint,
                           ctxp: *mut c_void,
                           malocfp: Option<extern "C" fn(ctxp: *mut c_void, size: c_ulong)
                                                         -> *mut c_void>,
                           ralocfp: Option<extern "C" fn(ctxp: c_void,
                                                         memptr: c_void,
                                                         newsize: c_ulong)
                                                         -> *mut c_void>,
                           mfreefp: Option<extern "C" fn(ctxp: *mut c_void, memptr: *mut c_void)>,
                           xtramem_sz: c_ulong,
                           usrmempp: *mut *mut c_void,
                           charset: c_ushort,
                           ncharset: c_ushort)
                           -> c_int;

    pub fn OCIHandleAlloc(parenth: *const c_void,
                          hndlpp: *mut *mut c_void,
                          _type: c_uint,
                          xtramem_sz: c_ulong,
                          usrmempp: *mut *mut c_void)
                          -> c_int;

    pub fn OCIServerAttach(srvhp: *mut OCIServer,
                           errhp: *mut OCIError,
                           dblink: *const c_uchar,
                           dblink_len: c_int,
                           mode: c_uint)
                           -> c_int;

    pub fn OCIAttrGet(trgthndlp: *const c_void,
                      trghndltyp: c_uint,
                      attributep: *mut c_void,
                      sizep: *mut c_uint,
                      attrtype: c_uint,
                      errhp: *mut OCIError)
                      -> c_int;

    pub fn OCIAttrSet(trgthndlp: *mut c_void,
                      trghndltyp: c_uint,
                      attributep: *mut c_void,
                      size: c_uint,
                      attrtype: c_uint,
                      errhp: *mut OCIError)
                      -> c_int;

    pub fn OCISessionBegin(svchp: *mut OCISvcCtx,
                           errhp: *mut OCIError,
                           usrhp: *mut OCISession,
                           credt: c_uint,
                           mode: c_uint)
                           -> c_int;

    pub fn OCISessionEnd(svchp: *mut OCISvcCtx,
                         errhp: *mut OCIError,
                         usrhp: *mut OCISession,
                         mode: c_uint)
                         -> c_int;

    pub fn OCIServerDetach(srvhp: *mut OCIServer, errhp: *mut OCIError, mode: c_uint) -> c_int;

    pub fn OCIHandleFree(hndlp: *mut c_void, _type: c_uint) -> c_int;

    pub fn OCIStmtPrepare2(svchp: *mut OCISvcCtx,
                           stmtp: *mut *mut OCIStmt,
                           errhp: *mut OCIError,
                           stmt: *const c_uchar,
                           stmt_len: c_uint,
                           key: *const c_uchar,
                           key_len: c_uint,
                           language: c_uint,
                           mode: c_uint)
                           -> c_int;

    pub fn OCIStmtExecute(svchp: *mut OCISvcCtx,
                          stmtp: *mut OCIStmt,
                          errhp: *mut OCIError,
                          iters: c_uint,
                          rowoff: c_uint,
                          snap_in: *const OCISnapshot,
                          snap_out: *mut OCISnapshot,
                          mode: c_uint)
                          -> c_int;

    pub fn OCIStmtRelease(stmtp: *mut OCIStmt,
                          errhp: *mut OCIError,
                          key: *const c_uchar,
                          key_len: c_uint,
                          mode: c_uint)
                          -> c_int;

    pub fn OCITransCommit(svchp: *mut OCISvcCtx, errhp: *mut OCIError, flags: c_uint) -> c_int;                          

    pub fn OCIParamGet(hndlp: *const c_void,
                       htype: c_uint,
                       errhp: *mut OCIError,
                       parmdpp: *mut *mut c_void,
                       pos: c_uint)
                       -> c_int;

    pub fn OCIDefineByPos(stmtp: *mut OCIStmt,
                          defnpp: *const *mut OCIDefine,
                          errhp: *mut OCIError,
                          position: c_uint,
                          valuep: *const u8,
                          value_sz: c_int,
                          dty: c_ushort,
                          indp: *const c_short,
                          rlenp: *const c_ushort,
                          rcodep: *const c_ushort,
                          mode: c_uint)
                          -> c_int;

    pub fn OCIBindByName(stmtp: *mut OCIStmt,
                         bindpp: *const *mut OCIBind,
                         errhp: *mut OCIError,
                         placeholder: *const c_uchar,
                         placeh_len: c_uint,
                         valuep: *const u8,
                         value_sz: c_int,
                         dty: c_ushort,
                         indp: *const c_short,
                         alenp: *const c_ushort,
                         rcodep: *const c_ushort,
                         maxarr_len: c_uint,
                         curelep: *const c_uint,
                         mode: c_uint)
                         -> c_int;

    pub fn OCIBindByPos(stmtp: *mut OCIStmt,
                        bindpp: *const *mut OCIBind,
                        errhp: *mut OCIError,
                        position: c_uint,
                        valuep: *const u8,
                        value_sz: c_int,
                        dty: c_ushort,
                        indp: *const c_short,
                        alenp: *const c_ushort,
                        rcodep: *const c_ushort,
                        maxarr_len: c_uint,
                        curelep: *const c_uint,
                        mode: c_uint)
                        -> c_int;

    pub fn OCIStmtFetch2(stmtp: *mut OCIStmt,
                         errhp: *mut OCIError,
                         nrows: c_uint,
                         orientation: c_ushort,
                         fetchOffset: c_int,
                         mode: c_uint)
                         -> c_int;

    pub fn OCIDescriptorAlloc(parenth: *const c_void,
                          descpp: *mut *mut c_void,
                          _type: c_uint,
                          xtramem_sz: c_ulong,
                          usrmempp: *mut *mut c_void)
                          -> c_int;

    pub fn OCIDescriptorFree(descp: *mut c_void, _type: c_uint) -> c_int;

    pub fn OCIArrayDescriptorAlloc(parenth: *const c_void,
                          descpp: *mut *mut c_void,
                          _type: c_uint,
                          array_size: c_uint,
                          xtramem_sz: c_ulong,
                          usrmempp: *mut *mut c_void)
                          -> c_int;

    pub fn OCIArrayDescriptorFree(descp: *mut *mut c_void, _type: c_uint) -> c_int;                          

    pub fn OCILobCreateTemporary(svchp: *mut OCISvcCtx,
                                 errhp: *mut OCIError,
                                 locp: *mut OCILobLocator,
                                 csid: c_ushort,
                                 csfrm: c_uchar,
                                 lobtype: c_uchar,
                                 cache: bool,
                                 duration: c_ushort) -> c_int;

    pub fn OCILobFreeTemporary(svchp: *mut OCISvcCtx,
                               errhp: *mut OCIError,
                               locp: *mut OCILobLocator) -> c_int;

    pub fn OCILobGetLength(svchp: *mut OCISvcCtx,
                           errhp: *mut OCIError,
                           locp: *mut OCILobLocator,
                           lenp: *const c_uint) -> c_int;  

    pub fn OCILobTrim(svchp: *mut OCISvcCtx,
                      errhp: *mut OCIError,
                      locp: *mut OCILobLocator,
                      newlen: c_uint) -> c_int;       

    pub fn OCILobCopy2(svchp: *mut OCISvcCtx,
                       errhp: *mut OCIError,
                       dst_locp: *mut OCILobLocator,
                       src_locp: *mut OCILobLocator,
                       amount: c_ulong,
                       dst_offset: c_ulong,
                       src_offset: c_ulong) -> c_int;     

    pub fn OCILobRead(svchp: *mut OCISvcCtx,
                      errhp: *mut OCIError,
                      locp: *mut OCILobLocator,
                      amtp: *const c_uint,
                      offset: c_uint,
                      bufp: *const u8,
                      bufl: c_uint,
                      ctxp: *const c_void,

                      cbfp: Option<extern "C" fn(ctxp: *const c_void,
                                                 bufp: *const u8,
                                                 len: c_uint,
                                                 piece: u8)
                                                 -> c_int>,

                       csid: c_ushort,
                       csfrm: u8
                       ) -> c_int;                     

    pub fn OCILobWrite(svchp: *mut OCISvcCtx,
                       errhp: *mut OCIError,
                       locp: *mut OCILobLocator,
                       amtp: *const c_uint,
                       offset: c_uint,
                       bufp: *const u8,
                       buflen: c_uint,
                       piece: u8,
                       ctxp: *const c_void,

                       cbfp: Option<extern "C" fn(ctxp: *const c_void,
                                                  bufp: *const u8,
                                                  lenp: *const c_uint,
                                                  piece: *const u8)
                                                  -> c_int>,

                        csid: c_ushort,
                        csfrm: u8
                        ) -> c_int;

}
