#![allow(dead_code)]

mod messages;

pub use messages::{
    ru_sktbelpa_pressure_self_writer_INFO_MAGICK,
    ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID,
    ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION,
    ru_sktbelpa_pressure_self_writer_Request, ru_sktbelpa_pressure_self_writer_Response,
    ru_sktbelpa_pressure_self_writer_STATUS_OK,
};

use nanopb_rs::pb_encode::get_encoded_size;
pub use nanopb_rs::pb_msgdesc_t;

static SIZE_ERROR_MSG: &str = "Failed to calculete message size";

pub trait Sizable<T> {
    fn get_size(data: &T) -> usize;
}

impl ru_sktbelpa_pressure_self_writer_Request {
    pub fn fields() -> &'static pb_msgdesc_t {
        unsafe { &messages::ru_sktbelpa_pressure_self_writer_Request_msg }
    }
}

impl Sizable<ru_sktbelpa_pressure_self_writer_Request>
    for ru_sktbelpa_pressure_self_writer_Request
{
    fn get_size(data: &ru_sktbelpa_pressure_self_writer_Request) -> usize {
        get_encoded_size(
            Self::fields(),
            data as *const ru_sktbelpa_pressure_self_writer_Request as *const ::core::ffi::c_void,
        )
        .map_err(|_| panic!("{}", SIZE_ERROR_MSG))
        .unwrap()
    }
}

impl ru_sktbelpa_pressure_self_writer_Response {
    pub fn fields() -> &'static pb_msgdesc_t {
        unsafe { &messages::ru_sktbelpa_pressure_self_writer_Response_msg }
    }
}

impl Sizable<ru_sktbelpa_pressure_self_writer_Response>
    for ru_sktbelpa_pressure_self_writer_Response
{
    fn get_size(data: &ru_sktbelpa_pressure_self_writer_Response) -> usize {
        get_encoded_size(
            Self::fields(),
            data as *const ru_sktbelpa_pressure_self_writer_Response as *const ::core::ffi::c_void,
        )
        .map_err(|_| panic!("{}", SIZE_ERROR_MSG))
        .unwrap()
    }
}
