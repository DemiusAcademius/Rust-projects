// use std::mem;
// use std::ptr;

// use super::oci::OCIDataType;
use super::meta::*;

impl MetaQuery for u32 {
    fn create(values: &ResultSet) -> u32 {
        let s0 = &(values[0]);
        s0.into()
    }

    fn meta() -> Vec<MetaType> {
        vec![ u32_meta ]
    }
}

impl MetaQuery for String {
    fn create(values: &ResultSet) -> String {
        let s0 = &(values[0]);
        s0.into()
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(200) ]
    }
}