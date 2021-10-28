#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::common::{pb_byte_t, pb_istream_t, pb_msgdesc_t, pb_wire_type_t, size_t};

include!("bindings/pb_decode.rs");

extern crate alloc;
use alloc::vec::Vec;

use crate::pb::Error;

pub struct IStream(pb_istream_t);

impl IStream {
    pub fn from_buffer(buf: &[u8]) -> Self {
        IStream {
            0: unsafe { pb_istream_from_buffer(buf.as_ptr(), buf.len()) },
        }
    }

    pub fn encode<T>(&mut self, fields: &pb_msgdesc_t) -> Result<T, Error> {
        let mut dest_struct: T = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
        if unsafe {
            pb_decode(
                &mut self.0,
                fields,
                &mut dest_struct as *mut T as *mut ::core::ffi::c_void,
            )
        } {
            Ok(dest_struct)
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_ex<T>(&mut self, fields: &pb_msgdesc_t, flags: u32) -> Result<T, Error> {
        let mut dest_struct: T = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
        if unsafe {
            pb_decode_ex(
                &mut self.0,
                fields,
                &mut dest_struct as *mut T as *mut ::core::ffi::c_void,
                flags,
            )
        } {
            Ok(dest_struct)
        } else {
            Err(self.get_error())
        }
    }

    pub fn read(&mut self, count: usize) -> Result<Vec<u8>, Error> {
        let mut buf: Vec<u8> = Vec::with_capacity(count);
        buf.resize(count, 0);
        if unsafe { pb_read(&mut self.0, buf.as_mut_ptr(), count) } {
            Ok(buf)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_tag(&mut self, wire_type: &mut pb_wire_type_t) -> Result<u32, Error> {
        let mut tag = 0_u32;
        let mut eof = false;
        if unsafe { pb_decode_tag(&mut self.0, wire_type, &mut tag, &mut eof) } {
            if eof {
                Err(Error::from_str("EOF\0")) // TODO
            } else {
                Ok(tag)
            }
        } else {
            Err(self.get_error())
        }
    }

    pub fn skip_field(&mut self, wire_type: pb_wire_type_t) -> Result<(), Error> {
        if unsafe { pb_skip_field(&mut self.0, wire_type) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_variant(&mut self) -> Result<u64, Error> {
        let mut res = 0_u64;
        if unsafe { pb_decode_varint(&mut self.0, &mut res) } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_variant32(&mut self) -> Result<u32, Error> {
        let mut res = 0_u32;
        if unsafe { pb_decode_varint32(&mut self.0, &mut res) } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_bool(&mut self) -> Result<bool, Error> {
        let mut res = false;
        if unsafe { pb_decode_bool(&mut self.0, &mut res) } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_svariant(&mut self) -> Result<i64, Error> {
        let mut res = 0_i64;
        if unsafe { pb_decode_svarint(&mut self.0, &mut res) } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_fixed32(&mut self) -> Result<u32, Error> {
        let mut res = 0_u32;
        if unsafe {
            pb_decode_fixed32(
                &mut self.0,
                &mut res as *mut u32 as *mut ::core::ffi::c_void,
            )
        } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_fixed64(&mut self) -> Result<u64, Error> {
        let mut res = 0_u64;
        if unsafe {
            pb_decode_fixed64(
                &mut self.0,
                &mut res as *mut u64 as *mut ::core::ffi::c_void,
            )
        } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    fn get_error(&self) -> Error {
        Error::new(self.0.errmsg)
    }
}
