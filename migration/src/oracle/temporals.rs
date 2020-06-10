use std::fmt;
use std::mem;

use super::meta::{ MetaType, ResultItem };
use oracle::chrono::*;
use super::oci::OCIDataType;

pub const date_meta: MetaType = MetaType { size: 7, align: 1, oci_type: OCIDataType::Date };
pub const timestamp_meta: MetaType = MetaType { size: 11, align: 1, oci_type: OCIDataType::Timestamp };

/*
pub fn date_meta() -> MetaType {
    MetaType { size: mem::size_of::<[u8; 7]>(), align: mem::align_of::<[u8; 7]>(), oci_type: OCIDataType::Date }
}

pub fn timestamp_meta() -> MetaType {
    MetaType { size: mem::size_of::<[u8; 11]>(), align: mem::align_of::<[u8; 11]>(), oci_type: OCIDataType::Timestamp }
}
*/

type SqlDate = naive::date::NaiveDate;
type SqlDateTime = naive::datetime::NaiveDateTime;
type SqlTImestamp = naive::time::NaiveTime;

impl<'a> From<&'a ResultItem> for SqlDate {
    fn from(result: &'a ResultItem) -> SqlDate {
        let vec = unsafe { mem::transmute::<*const u8, &[u8; 7]>(result.q).to_owned() };
        let y = (vec[0] as i32 - 100)*100 + vec[1] as i32 - 100;
        let m = vec[2] as u32;
        let d = vec[3] as u32;
        naive::date::NaiveDate::from_ymd(y, m, d)        
    }
} 

// TODO: datetime: idx 4, 5, 6

// TODO: timestamp: vec[11] etc