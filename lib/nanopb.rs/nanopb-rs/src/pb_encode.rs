#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use core::intrinsics::transmute;
use core::ptr::{null, slice_from_raw_parts};

extern crate alloc;

use alloc::boxed::Box;
use my_proc_macro::c_str;

pub use crate::common::{
    pb_byte_t, pb_field_iter_t, pb_msgdesc_t, pb_ostream_t, pb_wire_type_t, size_t,
};

include!("bindings/pb_encode.rs");

use crate::pb::Error;

pub trait tx_context {
    fn write(&mut self, buff: &[u8]) -> Result<usize, ()>;
}

impl tx_context for u8 {
    fn write(&mut self, _buff: &[u8]) -> Result<usize, ()> {
        unreachable!();
    }
}

pub struct OStream {
    ctx: pb_ostream_t,
    writer: Option<Box<dyn tx_context>>,
}

impl OStream {
    pub fn from_buffer(buf: &mut [u8]) -> Self {
        OStream {
            ctx: unsafe { pb_ostream_from_buffer(buf.as_mut_ptr(), buf.len()) },
            writer: None,
        }
    }

    pub fn from_callback<T: tx_context + 'static>(tx_ctx: T, max_size: Option<usize>) -> Self {
        unsafe extern "C" fn write_wraper<U: tx_context>(
            stream: *mut pb_ostream_t,
            buf: *const u8,
            count: usize,
        ) -> bool {
            let cb = transmute::<*mut ::core::ffi::c_void, *mut U>((*stream).state);
            match (*cb).write(&*slice_from_raw_parts(buf, count)) {
                Ok(writen) => writen == count,
                Err(_) => false,
            }
        }

        let mut res = OStream {
            ctx: crate::common::pb_ostream_s {
                callback: Some(write_wraper::<T>),
                state: null::<::core::ffi::c_void>() as *mut _,
                max_size: max_size.unwrap_or(usize::MAX),
                bytes_written: 0,
                errmsg: null(),
            },
            writer: Some(Box::new(tx_ctx)),
        };

        res.ctx.state = res.writer.as_ref().unwrap() as *const _ as *mut _;

        res
    }

    pub fn stram_size(&self) -> usize {
        self.ctx.bytes_written
    }

    pub fn stream(&mut self) -> &mut pb_ostream_t {
        &mut self.ctx
    }

    pub fn bytes_writen(&self) -> usize {
        self.ctx.bytes_written
    }
}

impl pb_ostream_t {
    pub fn encode<U>(&mut self, fields: &pb_msgdesc_t, src_struct: &U) -> Result<(), Error> {
        if unsafe {
            pb_encode(
                self,
                fields,
                src_struct as *const U as *const ::core::ffi::c_void,
            )
        } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_ex<U>(
        &mut self,
        fields: &pb_msgdesc_t,
        src_struct: &U,
        flags: u32,
    ) -> Result<(), Error> {
        if unsafe {
            pb_encode_ex(
                self,
                fields,
                src_struct as *const U as *const ::core::ffi::c_void,
                flags,
            )
        } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), Error> {
        if unsafe { pb_write(self, buf.as_ptr(), buf.len()) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_tag_for_field(&mut self, field: &pb_field_iter_t) -> Result<(), Error> {
        if unsafe { pb_encode_tag_for_field(self, field) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_tag(&mut self, wiretype: pb_wire_type_t, field_number: u32) -> Result<(), Error> {
        if unsafe { pb_encode_tag(self, wiretype, field_number) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_varint(&mut self, value: u64) -> Result<(), Error> {
        if unsafe { pb_encode_varint(self, value) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_svarint(&mut self, value: i64) -> Result<(), Error> {
        if unsafe { pb_encode_svarint(self, value) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_string(&mut self, s: &str) -> Result<(), Error> {
        if unsafe { pb_encode_string(self, s.as_ptr(), s.len()) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_fixed32(&mut self, value: u32) -> Result<(), Error> {
        if unsafe { pb_encode_fixed32(self, &value as *const u32 as *const _) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_fixed64(&mut self, value: u64) -> Result<(), Error> {
        if unsafe { pb_encode_fixed64(self, &value as *const u64 as *const _) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_f32(&mut self, value: f32) -> Result<(), Error> {
        self.encode_fixed32(unsafe { *(&value as *const f32 as *const u32) })
    }

    pub fn encode_f64(&mut self, value: f64) -> Result<(), Error> {
        self.encode_fixed64(unsafe { *(&value as *const f64 as *const u64) })
    }

    fn get_error(&self) -> Error {
        Error::new(self.errmsg)
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
        Err(Error::from_str(c_str!("Failed to calculule message size")))
    }
}
