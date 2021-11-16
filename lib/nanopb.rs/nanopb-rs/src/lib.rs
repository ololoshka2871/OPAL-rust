#![no_std]

mod common;
pub use common::pb_msgdesc_t;
pub use common::pb_callback_t;
pub use common::pb_size_t;

pub mod pb;
pub use pb::Error;

pub mod pb_encode;
pub use pb_encode::OStream;

pub mod pb_decode;
pub use pb_decode::IStream;

pub mod dyn_fields;