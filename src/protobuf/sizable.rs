use nanopb_rs::pb_encode::get_encoded_size;

use super::{ru_sktbelpa_pressure_self_writer_Request, ru_sktbelpa_pressure_self_writer_Response};

static SIZE_ERROR_MSG: &str = "Failed to calculete message size";

pub trait Sizable<T> {
    fn get_size(data: &T) -> usize;
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
