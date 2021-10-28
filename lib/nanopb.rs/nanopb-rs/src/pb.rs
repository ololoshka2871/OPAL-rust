#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use core::fmt::Display;

use cstr_core;

//include!("bindings/pb.rs");

pub struct Error {
    msg: *const u8,
}

impl Error {
    pub fn new(msg: *const u8) -> Self {
        Error { msg }
    }
    pub fn from_str(msg: &'static str) -> Self {
        Error { msg: msg.as_ptr() }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.msg.is_null() {
            f.write_str("None")
        } else {
            let s =  unsafe { cstr_core::CStr::from_ptr(self.msg) };
            f.write_str(s.to_str().unwrap())
        }
    }
}