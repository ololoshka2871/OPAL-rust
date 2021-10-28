#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::common::{
    pb_byte_t, pb_field_iter_t, pb_msgdesc_t, pb_ostream_t, pb_wire_type_t, size_t,
};

include!("bindings/pb_encode.rs");

use crate::pb::Error;

pub struct OStream(pb_ostream_t);

impl OStream {
    pub fn from_buffer(buf: &mut [u8]) -> Self {
        OStream {
            0: unsafe { pb_ostream_from_buffer(buf.as_mut_ptr(), buf.len()) },
        }
    }

    pub fn encode<T>(&mut self, fields: &pb_msgdesc_t, src_struct: &T) -> Result<(), Error> {
        if unsafe {
            pb_encode(
                &mut self.0,
                fields,
                src_struct as *const T as *const ::core::ffi::c_void,
            )
        } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_ex<T>(
        &mut self,
        fields: &pb_msgdesc_t,
        src_struct: &T,
        flags: u32,
    ) -> Result<(), Error> {
        if unsafe {
            pb_encode_ex(
                &mut self.0,
                fields,
                src_struct as *const T as *const ::core::ffi::c_void,
                flags,
            )
        } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        if unsafe { pb_write(&mut self.0, buf.as_ptr(), buf.len()) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_tag_for_field(&mut self, field: *const pb_field_iter_t) -> Result<(), Error> {
        if unsafe { pb_encode_tag_for_field(&mut self.0, field) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_tag(&mut self, wiretype: pb_wire_type_t, field_number: u32) -> Result<(), Error> {
        if unsafe { pb_encode_tag(&mut self.0, wiretype, field_number) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_varint(&mut self, value: u64) -> Result<(), Error> {
        if unsafe { pb_encode_varint(&mut self.0, value) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_svarint(&mut self, value: i64) -> Result<(), Error> {
        if unsafe { pb_encode_svarint(&mut self.0, value) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_string(&mut self, s: &str) -> Result<(), Error> {
        if unsafe { pb_encode_string(&mut self.0, s.as_ptr(), s.len()) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_fixed32(&mut self, value: u32) -> Result<(), Error> {
        if unsafe {
            pb_encode_fixed32(
                &mut self.0,
                &value as *const u32 as *const ::core::ffi::c_void,
            )
        } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_fixed64(&mut self, value: u64) -> Result<(), Error> {
        if unsafe {
            pb_encode_fixed64(
                &mut self.0,
                &value as *const u64 as *const ::core::ffi::c_void,
            )
        } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn get_encoded_size(
        fields: &pb_msgdesc_t,
        src_struct: *const ::core::ffi::c_void,
    ) -> Result<usize, Error> {
        let mut s = 0_usize;
        if unsafe { pb_get_encoded_size(&mut s, fields, src_struct) } {
            Ok(s)
        } else {
            Err(Error::from_str("Failed to calculule message size\0"))
        }
    }

    fn get_error(&self) -> Error {
        Error::new(self.0.errmsg)
    }

    pub fn bytes_writen(&self) -> usize {
        self.0.bytes_written    
    }
}
