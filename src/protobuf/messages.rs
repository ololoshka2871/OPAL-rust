#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use nanopb_rs::pb_msgdesc_t;

pub const _ru_sktbelpa_pressure_self_writer_INFO_ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION : _ru_sktbelpa_pressure_self_writer_INFO = 1 ;
pub const _ru_sktbelpa_pressure_self_writer_INFO_ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID : _ru_sktbelpa_pressure_self_writer_INFO = 57350 ;
pub const _ru_sktbelpa_pressure_self_writer_INFO_ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER : _ru_sktbelpa_pressure_self_writer_INFO = 65535 ;
pub const _ru_sktbelpa_pressure_self_writer_INFO_ru_sktbelpa_pressure_self_writer_INFO_MAGICK:
    _ru_sktbelpa_pressure_self_writer_INFO = 9;
pub type _ru_sktbelpa_pressure_self_writer_INFO = u32;
pub use self::_ru_sktbelpa_pressure_self_writer_INFO as ru_sktbelpa_pressure_self_writer_INFO;
pub const _ru_sktbelpa_pressure_self_writer_STATUS_ru_sktbelpa_pressure_self_writer_STATUS_OK:
    _ru_sktbelpa_pressure_self_writer_STATUS = 0;
pub const _ru_sktbelpa_pressure_self_writer_STATUS_ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS : _ru_sktbelpa_pressure_self_writer_STATUS = 1 ;
pub const _ru_sktbelpa_pressure_self_writer_STATUS_ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR : _ru_sktbelpa_pressure_self_writer_STATUS = 100 ;
pub type _ru_sktbelpa_pressure_self_writer_STATUS = u32;
pub use self::_ru_sktbelpa_pressure_self_writer_STATUS as ru_sktbelpa_pressure_self_writer_STATUS;
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_Request {
    pub id: u32,
    pub deviceID: u32,
    pub protocolVersion: u32,
}

pub type ru_sktbelpa_pressure_self_writer_Request = _ru_sktbelpa_pressure_self_writer_Request;
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_Response {
    pub id: u32,
    pub deviceID: u32,
    pub protocolVersion: u32,
    pub Global_status: ru_sktbelpa_pressure_self_writer_STATUS,
    pub timestamp: u64,
}

pub type ru_sktbelpa_pressure_self_writer_Response = _ru_sktbelpa_pressure_self_writer_Response;
extern "C" {
    pub static ru_sktbelpa_pressure_self_writer_Request_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_Response_msg: pb_msgdesc_t;
}
