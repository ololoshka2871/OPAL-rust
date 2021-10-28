#![no_std]

mod common;

pub mod pb;
pub use pb::Error;

pub mod pb_encode;
pub use pb_encode::OStream;

pub mod pb_decode;
pub use pb_decode::IStream;

pub use common::pb_msgdesc_t;