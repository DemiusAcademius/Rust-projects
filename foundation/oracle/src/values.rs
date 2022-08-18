use std::mem::transmute;
use std::ptr;

pub trait FromResultSet {
    fn from_resultset(rs: &ResultSet) -> Self;
}

pub enum SqlValue {
    Nil,
    Val { valp: *const u8, len: u16 }
}
pub type ResultSet = Vec<SqlValue>;

// Automatic convert SqlValue to rust types

impl SqlValue {

    #[inline]
    pub fn new(valp: *const u8, len: u16) -> SqlValue {
        SqlValue::Val {valp, len}
    }

    #[inline]
    pub fn map<U,F: FnOnce(*const u8, u16) -> U>(&self, f: F) -> Option<U> {
        match self {
            SqlValue::Val {valp, len} => Some(f(*valp,*len)),
            SqlValue::Nil => None
        }
    }

    #[inline]
    pub fn map_or<U,F: FnOnce(*const u8, u16) -> U>(&self, default: U, f: F) -> U {
        match self {
            SqlValue::Val {valp, len} => f(*valp,*len),
            SqlValue::Nil => default
        }
    }
}

// integer types, must be used only for primitive types

macro_rules! from_sql_to_primitive {
    ($T:ty) => {

        impl From<&SqlValue> for $T {
            fn from(v: &SqlValue) -> $T {
                v.map_or(Default::default(),|valp,_|unsafe { transmute::<*const u8, &$T>(valp) }.to_owned())
            }
        }

        impl From<&SqlValue> for Option<$T> {
            fn from(v: &SqlValue) -> Option<$T> {
                v.map(|valp,_|unsafe { transmute::<*const u8, &$T>(valp) }.to_owned())
            }
        }

    }
}

from_sql_to_primitive!(i16);
from_sql_to_primitive!(u16);

from_sql_to_primitive!(i32);
from_sql_to_primitive!(u32);

from_sql_to_primitive!(i64);
from_sql_to_primitive!(u64);

from_sql_to_primitive!(f64);

// String type, in Oracle NULL String is Empty String

impl From<&SqlValue> for String {
    fn from(v: &SqlValue) -> String {
        v.map_or(String::new(),|valp,len| {
            let str_len = len as usize;
            let mut dst = Vec::with_capacity(str_len) as Vec<u8>;
            unsafe {
                dst.set_len(str_len);
                ptr::copy(valp, dst.as_mut_ptr(), str_len);
                String::from_utf8_unchecked(dst)
            }
        })
    }
}

// boolean type mapped to u16 (INT TYPE IN DB), NULL is False
impl From<&SqlValue> for bool {
    fn from(v: &SqlValue) -> bool {
        v.map_or(false,|valp,_| unsafe { transmute::<*const u8, &bool>(valp) }.to_owned())
    }
}

// impl metainfo for singular primitive types

macro_rules! impl_from_resultset {
    ($T:ty) => {

        impl FromResultSet for $T {
            fn from_resultset(rs: &ResultSet) -> Self {
                let s0 = &(rs[0]);
                s0.into()
            }
        }

    }
}

impl_from_resultset!(u32);
impl_from_resultset!(i32);
impl_from_resultset!(bool);

impl FromResultSet for String {
    fn from_resultset(rs: &ResultSet) -> Self {
        let s0 = &(rs[0]);
        s0.into()
    }
}

// Date and Datetime
use chrono::prelude::*;
use crate::dates::*;

// TODO: Datetime have 7 bytes
// TODO: Timestamp have 11 bytes

impl From<&SqlValue> for SqlDate {
    fn from(v: &SqlValue) -> SqlDate {
        v.map_or(Local::now().date(),|valp,len| {
            assert!(len == 7, "Oracle Date length must be 7 bypes");
            let vec = unsafe { transmute::<*const u8, &[u8; 7]>(valp) };

            let y = (vec[0] as i32 - 100)*100 + vec[1] as i32 - 100;
            let m = vec[2] as u32;
            let d = vec[3] as u32;

            Local.ymd(y,m,d)
        })
    }
}

impl From<&SqlValue> for SqlDateTime {
    fn from(v: &SqlValue) -> SqlDateTime {
        v.map_or(Local::now(),|valp,len| {
            assert!(len == 11, "Oracle Date length must be 11 bypes");
            let vec = unsafe { transmute::<*const u8, &[u8; 11]>(valp) };

            let y = (vec[0] as i32 - 100)*100 + vec[1] as i32 - 100;
            let m = vec[2] as u32;
            let d = vec[3] as u32;

            let hh = vec[4] as u32;
            let mm = vec[5] as u32;
            let ss = vec[6] as u32;

            Local.ymd(y,m,d).and_hms(hh,mm,ss)
        })
    }
}

impl_from_resultset!(SqlDate);
impl_from_resultset!(SqlDateTime);

// TODO: optional converters for date and datetime
