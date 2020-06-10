use std::mem;
use std::ptr;
use std::slice;
use std::str;

use super::oci::OCIDataType;

/*
struct TableStruct (String, String, String);
impl MetaQuery for TableStruct {
    fn create(values: &ResultSet) -> TableStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        TableStruct(s0.into(), s1.into(), s2.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(100), string_meta(1), string_meta(3) ]
    }
}        
*/

#[derive(Debug)]
pub struct MetaType {
    pub size: u32,
    pub align: u32,
    pub oci_type: OCIDataType
}

pub const u32_meta: MetaType = MetaType { size: 4, align: 4, oci_type: OCIDataType::Uint };
pub const i32_meta: MetaType = MetaType { size: 4, align: 4, oci_type: OCIDataType::Int };

pub const u64_meta: MetaType = MetaType { size: 8, align: 8, oci_type: OCIDataType::Uint };

pub const f64_meta: MetaType = MetaType { size: 8, align: 8, oci_type: OCIDataType::Float };

pub fn string_meta(capacity: u32) -> MetaType {
    let char_size = mem::align_of::<char>() as u32;
    MetaType { size: capacity * char_size + 2, align: char_size, oci_type: OCIDataType::Char }
}

pub const longchar_meta: MetaType = MetaType { size: 16000, align: 16000, oci_type: OCIDataType::Long };

/*
pub fn u32_meta() -> MetaType {
    MetaType { size: mem::size_of::<u32>(), align: mem::align_of::<u32>(), oci_type: OCIDataType::Uint }
}

pub fn i32_meta() -> MetaType {
    MetaType { size: mem::size_of::<i32>(), align: mem::align_of::<i32>(), oci_type: OCIDataType::Int }
}

pub fn f64_meta() -> MetaType {
    MetaType { size: mem::size_of::<f64>(), align: mem::align_of::<f64>(), oci_type: OCIDataType::Float }
}
*/

pub trait MetaQuery {
    fn create(&ResultSet) -> Self;
    fn meta() -> Vec<MetaType>;
}

pub type ResultSet = Vec<ResultItem>;

pub struct ResultItem {
    pub q:         *const u8,
    pub indicator: bool,
    pub len:       usize
}

impl ResultItem {
    pub fn new(q: *const u8, indicator: bool, len: usize) -> ResultItem {
        ResultItem { q: q, indicator: indicator, len: len }
    }
}

impl<'a> From<&'a ResultItem> for u32 {
    fn from(result: &'a ResultItem) -> u32 {
        unsafe {
            mem::transmute::<*const u8, &u32>(result.q).to_owned()
        }
    }
} 

impl<'a> From<&'a ResultItem> for u64 {
    fn from(result: &'a ResultItem) -> u64 {
        unsafe {
            mem::transmute::<*const u8, &u64>(result.q).to_owned()
        }
    }
}

impl<'a>  From<&'a ResultItem> for Option<u32> {
    fn from(result: &'a ResultItem) -> Option<u32> {
        if result.indicator {
            unsafe {
                Some( mem::transmute::<*const u8, &u32>(result.q).to_owned() )
            }
        } else {
            None
        }
    }
} 

impl<'a> From<&'a ResultItem> for i32 {
    fn from(result: &'a ResultItem) -> i32 {
        unsafe {
            mem::transmute::<*const u8, &i32>(result.q).to_owned()
        }
    }
} 

impl<'a> From<&'a ResultItem> for f64 {
    fn from(result: &'a ResultItem) -> f64 {
        unsafe {
            mem::transmute::<*const u8, &f64>(result.q).to_owned()
        }
    }
} 

impl<'a>  From<&'a ResultItem> for Option<f64> {
    fn from(result: &'a ResultItem) -> Option<f64> {
        if result.indicator {
            unsafe {
                Some( mem::transmute::<*const u8, &f64>(result.q).to_owned() )
            }
        } else {
            None
        }
    }
} 

impl<'a>  From<&'a ResultItem> for String {
    fn from(result: &'a ResultItem) -> String {
        let mut dst = Vec::with_capacity(result.len) as Vec<u8>;
        unsafe {
            dst.set_len(result.len); 
            ptr::copy(result.q, dst.as_mut_ptr(), result.len);
            // String::from_raw_parts(result.q as *mut _, result.len, result.len).clone()
            String::from_utf8_unchecked(dst)
        }
    }
} 

impl<'a>  From<&'a ResultItem> for Option<String> {
    fn from(result: &'a ResultItem) -> Option<String> {
        if result.indicator {
            let mut dst = Vec::with_capacity(result.len) as Vec<u8>;
            unsafe {
                dst.set_len(result.len); 
                ptr::copy(result.q, dst.as_mut_ptr(), result.len);
                Some(String::from_utf8_unchecked(dst))
            }
        } else {
            None
        }
        
    }
} 

impl<'a>  From<&'a ResultItem> for &'a str {
    fn from(result: &'a ResultItem) -> &'a str {
        unsafe {
            let slice = slice::from_raw_parts(result.q, result.len);
            str::from_utf8_unchecked(slice) 
        }
    }
} 