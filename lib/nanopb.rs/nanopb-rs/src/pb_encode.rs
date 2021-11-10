#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use core::intrinsics::transmute;
use core::ptr::{null, slice_from_raw_parts};

extern crate alloc;

use alloc::boxed::Box;

pub use crate::common::{
    pb_byte_t, pb_field_iter_t, pb_msgdesc_t, pb_ostream_s, pb_ostream_t, pb_wire_type_t, size_t,
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
            stream: *mut pb_ostream_s,
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

    pub fn encode<U>(&mut self, fields: &pb_msgdesc_t, src_struct: &U) -> Result<(), Error> {
        if unsafe {
            pb_encode(
                &mut self.ctx,
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
                &mut self.ctx,
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
        if unsafe { pb_write(&mut self.ctx, buf.as_ptr(), buf.len()) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_tag_for_field(&mut self, field: *const pb_field_iter_t) -> Result<(), Error> {
        if unsafe { pb_encode_tag_for_field(&mut self.ctx, field) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_tag(&mut self, wiretype: pb_wire_type_t, field_number: u32) -> Result<(), Error> {
        if unsafe { pb_encode_tag(&mut self.ctx, wiretype, field_number) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_varint(&mut self, value: u64) -> Result<(), Error> {
        if unsafe { pb_encode_varint(&mut self.ctx, value) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_svarint(&mut self, value: i64) -> Result<(), Error> {
        if unsafe { pb_encode_svarint(&mut self.ctx, value) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_string(&mut self, s: &str) -> Result<(), Error> {
        if unsafe { pb_encode_string(&mut self.ctx, s.as_ptr(), s.len()) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn encode_fixed32(&mut self, value: u32) -> Result<(), Error> {
        if unsafe {
            pb_encode_fixed32(
                &mut self.ctx,
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
                &mut self.ctx,
                &value as *const u64 as *const ::core::ffi::c_void,
            )
        } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    fn get_error(&self) -> Error {
        Error::new(self.ctx.errmsg)
    }

    pub fn bytes_writen(&self) -> usize {
        self.ctx.bytes_written
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