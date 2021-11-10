#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use core::{fmt::Debug};

pub type size_t = usize;
pub type pb_byte_t = u8;
pub type pb_type_t = u8;
pub type pb_size_t = u16;
pub type pb_ssize_t = i16;

pub type pb_istream_t = pb_istream_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pb_istream_s {
    pub callback: ::core::option::Option<
        unsafe extern "C" fn(stream: *mut pb_istream_t, buf: *mut pb_byte_t, count: size_t) -> bool,
    >,
    pub state: *mut ::core::ffi::c_void,
    pub bytes_left: size_t,
    pub errmsg: *const u8,
}

pub type pb_ostream_t = pb_ostream_s;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pb_ostream_s {
    pub callback: ::core::option::Option<
        unsafe extern "C" fn(
            stream: *mut pb_ostream_t,
            buf: *const pb_byte_t,
            count: size_t,
        ) -> bool,
    >,
    pub state: *mut ::core::ffi::c_void,
    pub max_size: size_t,
    pub bytes_written: size_t,
    pub errmsg: *const u8,
}

pub type pb_msgdesc_t = pb_msgdesc_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pb_msgdesc_s {
    pub field_info: *const u32,
    pub submsg_info: *const *const pb_msgdesc_t,
    pub default_value: *const pb_byte_t,
    pub field_callback: ::core::option::Option<
        unsafe extern "C" fn(
            istream: *mut pb_istream_t,
            ostream: *mut pb_ostream_t,
            field: *const pb_field_iter_t,
        ) -> bool,
    >,
    pub field_count: pb_size_t,
    pub required_field_count: pb_size_t,
    pub largest_tag: pb_size_t,
}

pub type pb_field_iter_t = pb_field_iter_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pb_field_iter_s {
    pub descriptor: *const pb_msgdesc_t,
    pub message: *mut ::core::ffi::c_void,
    pub index: pb_size_t,
    pub field_info_index: pb_size_t,
    pub required_field_index: pb_size_t,
    pub submessage_index: pb_size_t,
    pub tag: pb_size_t,
    pub data_size: pb_size_t,
    pub array_size: pb_size_t,
    pub type_: pb_type_t,
    pub pField: *mut ::core::ffi::c_void,
    pub pData: *mut ::core::ffi::c_void,
    pub pSize: *mut ::core::ffi::c_void,
    pub submsg_desc: *const pb_msgdesc_t,
}

pub type pb_field_t = pb_field_iter_t;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pb_bytes_array_s {
    pub size: pb_size_t,
    pub bytes: [pb_byte_t; 1usize],
}
pub type pb_bytes_array_t = pb_bytes_array_s;
pub type pb_callback_t = pb_callback_s;
#[repr(C)]

pub struct pb_callback_s {
    pub funcs: pb_callback_s__bindgen_ty_1,
    pub arg: *mut ::core::ffi::c_void,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union pb_callback_s__bindgen_ty_1 {
    pub decode: ::core::option::Option<
        unsafe extern "C" fn(
            stream: *mut pb_istream_t,
            field: *const pb_field_t,
            arg: *mut *mut ::core::ffi::c_void,
        ) -> bool,
    >,
    pub encode: ::core::option::Option<
        unsafe extern "C" fn(
            stream: *mut pb_ostream_t,
            field: *const pb_field_t,
            arg: *const *mut ::core::ffi::c_void,
        ) -> bool,
    >,
}

pub const pb_wire_type_t_PB_WT_VARINT: pb_wire_type_t = 0;
pub const pb_wire_type_t_PB_WT_64BIT: pb_wire_type_t = 1;
pub const pb_wire_type_t_PB_WT_STRING: pb_wire_type_t = 2;
pub const pb_wire_type_t_PB_WT_32BIT: pb_wire_type_t = 5;
pub const pb_wire_type_t_PB_WT_PACKED: pb_wire_type_t = 255;
pub type pb_wire_type_t = u32;
pub type pb_extension_type_t = pb_extension_type_s;
pub type pb_extension_t = pb_extension_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pb_extension_type_s {
    pub decode: ::core::option::Option<
        unsafe extern "C" fn(
            stream: *mut pb_istream_t,
            extension: *mut pb_extension_t,
            tag: u32,
            wire_type: pb_wire_type_t,
        ) -> bool,
    >,
    pub encode: ::core::option::Option<
        unsafe extern "C" fn(stream: *mut pb_ostream_t, extension: *const pb_extension_t) -> bool,
    >,
    pub arg: *const ::core::ffi::c_void,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct pb_extension_s {
    pub type_: *const pb_extension_type_t,
    pub dest: *mut ::core::ffi::c_void,
    pub next: *mut pb_extension_t,
    pub found: bool,
}

extern "C" {
    pub fn pb_default_field_callback(
        istream: *mut pb_istream_t,
        ostream: *mut pb_ostream_t,
        field: *const pb_field_t,
    ) -> bool;
}

impl Debug for pb_callback_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct Hex(usize);
        impl core::fmt::Debug for Hex {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "0x{:08x}", self.0)
            }
        }

        let addr = match unsafe { self.funcs.decode } {
            Some(f) => f as *const ::core::ffi::c_void as usize,
            None => 0
        };

        f.debug_struct("pb_callback_s")
            .field("funcs", &Hex(addr))
            .field("arg", &self.arg)
            .finish()
    }
}

impl Default for pb_callback_t {
    fn default() -> Self {
        Self {
            funcs: pb_callback_s__bindgen_ty_1 { decode: None },
            arg: ::core::ptr::null::<*const ::core::ffi::c_void>() as *mut _,
        }
    }
}
