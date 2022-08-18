extern crate libc;

use std::marker::PhantomData;
use std::mem;
use std::ptr;

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;

use super::oci::OCIDataType;

/// Bindigns for Cursor, Query and RowQuery
/// example: let mut id = bind! (5);
///          let mut nume = bind!("abracadabra".to_string( ))'
///          let mut query = conn.prepare_query("select ...", bindings!{"id" => id, "nume" => nume}, 100) ...
///          id.set(10); ... nume.set(another string); 

#[macro_export]
macro_rules! bindmap {
    ( $( $key:expr => $value:expr ),* ) => {
        {
            let mut temp_bindmap = Bindmap::new();
            $(
                temp_bindmap.insert($key, $value.binding.clone());
            )*
            Some(temp_bindmap)
        }
    };
}

#[macro_export]
macro_rules! bind {
    ( $value:expr ) => ( Binding::from($value); );
    ( $value:expr; $size:expr ) => ( Binding::new($value,$size); )
}

pub type Bindmap<'a> = HashMap<&'a str, Rc<RefCell<RowBinding>>>;
// pub type Bindmap<'a> = HashMap<&'a str, RowBinding>;

pub struct RowBinding {
    pub oci_type: OCIDataType,

    pub value_p: *const u8,
    pub indic_p: *const i16,

    pub max_size: u32,
    pub actual_size: u16 
}

pub struct Binding<T> {
    pub binding: Rc<RefCell<RowBinding>>,
    phantom:     PhantomData<T>
} 

impl RowBinding {

    pub fn new(oci_type: OCIDataType, value_p: *const u8, indicator: i16, max_size: u32, actual_size: u16) -> RowBinding {
        let indicator_p = unsafe { libc::malloc(2) as *const i16 };

        unsafe {
            ptr::copy(&indicator, indicator_p as *mut i16, 1);
        }

        RowBinding { 
            oci_type: oci_type,
            value_p: value_p, indic_p: indicator_p,
            max_size:  max_size, actual_size: actual_size
        }
    }

}

impl Binding<String> {

    pub fn new(v: &str, max_size: usize) -> Binding<String> {        
        let len = v.len();
        assert!(max_size >= len);
        let value_p = unsafe { libc::malloc(max_size) as *const u8 };

        unsafe {
            ptr::copy(v.as_ptr(), value_p as *mut u8, len);
        }
        
        Binding { 
            binding: Rc::new( RefCell::new(RowBinding::new(OCIDataType::Char, value_p, 0, max_size as u32, len as u16)) ),
            phantom: PhantomData
        }
    }

    pub fn set(&self, v: &str) {
        // println!("set string {}", v);
        let len = v.len() as u32;
        {
            let b = self.binding.borrow();
            assert!(b.max_size >= len, "max_size {} must be >= new size {}", b.max_size, len);        
        }
        unsafe {
            let mut b = self.binding.borrow_mut();
            // println!("   in unsafe");
            // b.actual_size = len;
            // b.value.
            let ptr_len: *mut u32 = mem::transmute(&b.actual_size);
            ptr::copy(&len, ptr_len, 1);
            ptr::copy(v.as_ptr(), b.value_p as *mut u8, len as usize);
        }
        // println!("actual_size: {} for len: {}", self.binding.borrow().actual_size, len);
    }
    
}

/*
impl<T> From<Binding<T>> for RowBinding {
    fn from(v: Binding<T>) -> RowBinding {
        v.binding
    }
} 
*/

impl From<u32> for Binding<u32> {
    fn from(v: u32) -> Binding<u32> {
        let value_p = unsafe { libc::malloc(4) as *const u8 };

        unsafe {
            ptr::copy(&v, value_p as *mut u32, 1);
        }
        
        Binding { 
            binding: Rc::new( RefCell::new(RowBinding::new(OCIDataType::Uint, value_p, 0, 4, 4)) ),
            // binding: RowBinding::new(OCIDataType::Uint, value_p, 0, 4, 4),
            phantom: PhantomData
        }
    }
} 

impl From<Option<u32>> for Binding<Option<u32>> {
    fn from(val: Option<u32>) -> Binding<Option<u32>> {
        let value_p = unsafe { libc::malloc(4) as *const u8 };

        let indicator = if let Some(v) = val {
            unsafe {
                ptr::copy(&v, value_p as *mut u32, 1);
            }
            0 
        } else { 
            -1
        };

        Binding { 
            binding: Rc::new( RefCell::new(RowBinding::new(OCIDataType::Uint, value_p, indicator, 4, 4)) ),
            // binding: RowBinding::new(OCIDataType::Uint, value_p, indicator, 4, 4),
            phantom: PhantomData
        }
    }
} 

impl From<String> for Binding<String> {
    fn from(v: String) -> Binding<String> {
        let len = v.len();
        let value_p = unsafe { libc::malloc(len) as *const u8 };

        unsafe {
            ptr::copy(v.as_ptr(), value_p as *mut u8, len);
        }
        
        Binding { 
            binding: Rc::new( RefCell::new(RowBinding::new(OCIDataType::Char, value_p, 0, len as u32, len as u16)) ),
            // binding: RowBinding::new(OCIDataType::Char, value_p, 0, len as u32, len as u32),
            phantom: PhantomData
        }
    }
}

impl <'a> From<&'a str> for Binding<String> {
    fn from(v: &str) -> Binding<String> {
        let len = v.len();
        let value_p = unsafe { libc::malloc(len) as *const u8 };

        unsafe {
            ptr::copy(v.as_ptr(), value_p as *mut u8, len);
        }
        
        Binding { 
            binding: Rc::new( RefCell::new(RowBinding::new(OCIDataType::Char, value_p, 0, len as u32, len as u16)) ),
            // binding: RowBinding::new(OCIDataType::Char, value_p, 0, len as u32, len as u32),
            phantom: PhantomData
        }
    }
}

impl <'a> From<&'a String> for Binding<String> {
    fn from(v: &String) -> Binding<String> {
        let len = v.len();
        let value_p = unsafe { libc::malloc(len) as *const u8 };

        unsafe {
            ptr::copy(v.as_ptr(), value_p as *mut u8, len);
        }
        
        Binding { 
            binding: Rc::new( RefCell::new(RowBinding::new(OCIDataType::Char, value_p, 0, len as u32, len as u16)) ),
            // binding: RowBinding::new(OCIDataType::Char, value_p, 0, len as u32, len as u32),
            phantom: PhantomData
        }
    }
} 
  
impl Binding<u32> {
    pub fn set(&self, v: u32) {
        let b = self.binding.borrow_mut();
        unsafe {
            ptr::copy(&v, b.value_p as *mut u32, 1);
        }
    }
}

impl Binding<Option<u32>> {
    pub fn set(&self, val: Option<u32>) {
        let mut b = self.binding.borrow_mut();
        let indicator = if let Some(v) = val {
            unsafe {
                ptr::copy(&v, b.value_p as *mut u32, 1);
            }
            0 
        } else { 
            -1
        };

        unsafe {
            ptr::copy(&indicator, b.indic_p as *mut i16, 1);
        }
        
    }
}

impl Drop for RowBinding {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.value_p as *mut libc::c_void);
            libc::free(self.indic_p as *mut libc::c_void);
        }
    }
}