use super::libc;

use super::oci;

use super::conn;

use std::boxed::Box;
use std::ptr;

pub struct LobDescriptor<'a> {
    conn:        &'a conn::Connection,
    pub locator: Box<*mut oci::OCILobLocator>,
    temporary:   bool,
    indicator:   *const libc::c_short
}

impl <'a> LobDescriptor <'a> {
    pub fn new(conn: &'a conn::Connection, temporary: bool, lobtype: oci::OCITempLobType, indicator: *const libc::c_short) -> Result<LobDescriptor<'a>, oci::OracleError> {
        let locator = Box::new( oci::oci_descriptor_alloc(conn.env.handle, oci::OCIDescriptorType::Lob)? as *mut oci::OCILobLocator );
        if temporary {
            oci::oci_lob_create_temporary(conn.service_handle, conn.env.error_handle, *locator, lobtype)?;
        }
        // let indicator = ptr::null();
        Ok( LobDescriptor { conn, locator, temporary, indicator } )
    }

    pub fn len(&self) -> Result<u32, oci::OracleError> {
        match oci::oci_lob_get_length(self.conn.service_handle, self.conn.env.error_handle, *self.locator) {
            Ok(len) => Ok(len),
            Err(error) => if error.code == -2 {
                // println!("oci_lob_get_length: invalid handle"); 
                Ok(0)
            } else {
                Err(error)
            }
        }         
    }

    pub fn is_null(&self) -> bool {
        if self.indicator.is_null() {
            true
        } else {
            let indicator = unsafe { *self.indicator };
            indicator < 0
        }
    }

    pub fn set_null(&mut self) {
        if self.indicator.is_null() {
            return;
        }
        unsafe {
            let indicator = 0;
            ptr::copy(&indicator, self.indicator as *mut i16, 1);
        }
    }
    
    /// read from lob to buffer
    /// return Ok(finita, rows) sau Err(error), finita -> true, end of read, false -> need data
    pub fn read(&mut self, offset: u32,buffer: *const u8, buffer_len: u32) -> Result<(bool, u32), oci::OracleError> {
        oci::oci_lob_read(self.conn.service_handle,
                                self.conn.env.error_handle,
                                *self.locator,
                                offset,
                                buffer,
                                buffer_len)
    }

    /// write from buffer to lob
    pub fn write(&mut self, write_len: u32, piece: oci::OCILobPiece, offset: u32, buffer: *const u8, buffer_len: u32) -> Result<(), oci::OracleError> {
        match oci::oci_lob_write(self.conn.service_handle,
                                 self.conn.env.error_handle,
                                 *self.locator,
                                 write_len,
                                 offset,
                                 buffer,
                                 buffer_len,
                                 piece) {
          Ok(_) => Ok(()),
          Err(error) => 
            if error.code == 99 {
                // OCI_NEED_DATA (code 99)
                if piece == oci::OCILobPiece::FirstPiece || piece == oci::OCILobPiece::NextPiece { Ok(()) } else { Err(error) }                         
            } else { Err(error) }
        }
    }

    pub fn trim(&mut self, total_lob_len: u32) -> Result<(), oci::OracleError> {
        oci::oci_lob_trim(self.conn.service_handle, self.conn.env.error_handle, *self.locator, total_lob_len)
    }

}

impl <'a> Drop for LobDescriptor <'a> {
    fn drop(&mut self) {
        if self.temporary {
            oci::oci_lob_free_temporary(self.conn.service_handle,
                                        self.conn.env.error_handle,
                                        *self.locator).ok().expect("oci_lob_free_temporary failed");
        }
        oci::oci_descriptor_free(*self.locator as *mut oci::c_void,
                                 oci::OCIDescriptorType::Lob)
        .ok().expect("oci_descriptor_free (lob_locator_descriptor) failed");
    }
}

pub fn lob_copy(src: &mut LobDescriptor, dst: &mut LobDescriptor, buffer: &[u8]) -> Result<u32, oci::OracleError> {
    let mut total_lob_len: u32 = 0;

    let buf_len = buffer.len() as u32;
    let buf_ptr = buffer.as_ptr();

    let mut is_first = true;
    let mut offset: u32 = 1;
    loop {
        let (finita, rows) = 
            match src.read(offset, buf_ptr, buf_len) {
                Ok((finita, rows)) => (finita, rows),
                Err(error) => {
                    break;        
                }
            };
        if rows == 0 {
            break;
        }

        let piece = if finita {
            if is_first { oci::OCILobPiece::OnePiece } else { oci::OCILobPiece::LastPiece }
        } else {
            if is_first { oci::OCILobPiece::FirstPiece } else { oci::OCILobPiece::NextPiece }
        };

        dst.write(rows, piece, offset, buf_ptr, buf_len)?;

        total_lob_len += rows;

        if finita {
            break;
        }

        is_first = false;
    }

    dst.trim(total_lob_len)?;
    Ok(total_lob_len)
}