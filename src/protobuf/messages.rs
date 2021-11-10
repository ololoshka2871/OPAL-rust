#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub use nanopb_rs::pb_callback_t;
use nanopb_rs::pb_msgdesc_t;

pub const ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION:
    _ru_sktbelpa_pressure_self_writer_INFO = 1;
pub const ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID:
    _ru_sktbelpa_pressure_self_writer_INFO = 57350;
pub const ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER:
    _ru_sktbelpa_pressure_self_writer_INFO = 65535;
pub const ru_sktbelpa_pressure_self_writer_INFO_MAGICK: u8 = 9;
pub type _ru_sktbelpa_pressure_self_writer_INFO = u32;
pub use self::_ru_sktbelpa_pressure_self_writer_INFO as ru_sktbelpa_pressure_self_writer_INFO;

pub type _ru_sktbelpa_pressure_self_writer_STATUS = u32;
pub use self::_ru_sktbelpa_pressure_self_writer_STATUS as ru_sktbelpa_pressure_self_writer_STATUS;
pub const ru_sktbelpa_pressure_self_writer_STATUS_OK: _ru_sktbelpa_pressure_self_writer_STATUS = 0;
pub const ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS:
    _ru_sktbelpa_pressure_self_writer_STATUS = 1;
pub const ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR:
    _ru_sktbelpa_pressure_self_writer_STATUS = 100;

//----------------------------------------------------------------------------------------------------

#[repr(C)]
#[derive(Debug, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_PCoefficientsGet {
    pub Fp0: f32,
    pub Ft0: f32,
    pub A: pb_callback_t,
}
pub type ru_sktbelpa_pressure_self_writer_PCoefficientsGet =
    _ru_sktbelpa_pressure_self_writer_PCoefficientsGet;

#[repr(C)]
#[derive(Debug, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_PCoefficientsSet {
    pub has_Fp0: bool,
    pub Fp0: f32,
    pub has_Ft0: bool,
    pub Ft0: f32,
    pub A: pb_callback_t,
}

pub type ru_sktbelpa_pressure_self_writer_PCoefficientsSet =
    _ru_sktbelpa_pressure_self_writer_PCoefficientsSet;

#[repr(C)]
#[derive(Debug, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_T5CoefficientsGet {
    pub T0: f32,
    pub F0: f32,
    pub C: pb_callback_t,
}
pub type ru_sktbelpa_pressure_self_writer_T5CoefficientsGet =
    _ru_sktbelpa_pressure_self_writer_T5CoefficientsGet;

#[repr(C)]
#[derive(Debug, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_T5CoefficientsSet {
    pub has_T0: bool,
    pub T0: f32,
    pub has_F0: bool,
    pub F0: f32,
    pub C: pb_callback_t,
}
pub type ru_sktbelpa_pressure_self_writer_T5CoefficientsSet =
    _ru_sktbelpa_pressure_self_writer_T5CoefficientsSet;

#[repr(C)]
#[derive(Debug, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_SettingsResponse {
    pub Serial: u32,
    pub PMesureTime_ms: u32,
    pub TMesureTime_ms: u32,
    pub Fref: u32,
    pub PEnabled: bool,
    pub TEnabled: bool,
    pub PCoefficients: ru_sktbelpa_pressure_self_writer_PCoefficientsGet,
    pub TCoefficients: ru_sktbelpa_pressure_self_writer_T5CoefficientsGet,
}
pub type ru_sktbelpa_pressure_self_writer_SettingsResponse =
    _ru_sktbelpa_pressure_self_writer_SettingsResponse;

#[repr(C)]
#[derive(Debug)]
pub struct _ru_sktbelpa_pressure_self_writer_WriteSettingsReq {
    pub has_Serial: bool,
    pub Serial: u32,
    pub has_PMesureTime_ms: bool,
    pub PMesureTime_ms: u32,
    pub has_TMesureTime_ms: bool,
    pub TMesureTime_ms: u32,
    pub has_Fref: bool,
    pub Fref: u32,
    pub has_PEnabled: bool,
    pub PEnabled: bool,
    pub has_TEnabled: bool,
    pub TEnabled: bool,
    pub has_PCoefficients: bool,
    pub PCoefficients: ru_sktbelpa_pressure_self_writer_PCoefficientsSet,
    pub has_TCoefficients: bool,
    pub TCoefficients: ru_sktbelpa_pressure_self_writer_T5CoefficientsSet,
}
pub type ru_sktbelpa_pressure_self_writer_WriteSettingsReq =
    _ru_sktbelpa_pressure_self_writer_WriteSettingsReq;

#[repr(C)]
#[derive(Debug)]
pub struct _ru_sktbelpa_pressure_self_writer_Request {
    pub id: u32,
    pub deviceID: u32,
    pub protocolVersion: u32,
    pub has_writeSettings: bool,
    pub writeSettings: ru_sktbelpa_pressure_self_writer_WriteSettingsReq,
}
pub type ru_sktbelpa_pressure_self_writer_Request = _ru_sktbelpa_pressure_self_writer_Request;

#[repr(C)]
#[derive(Debug, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_Response {
    pub id: u32,
    pub deviceID: u32,
    pub protocolVersion: u32,
    pub Global_status: ru_sktbelpa_pressure_self_writer_STATUS,
    pub timestamp: u64,
    pub has_getSettings: bool,
    pub getSettings: ru_sktbelpa_pressure_self_writer_SettingsResponse,
}
pub type ru_sktbelpa_pressure_self_writer_Response = _ru_sktbelpa_pressure_self_writer_Response;

extern "C" {
    pub static ru_sktbelpa_pressure_self_writer_Request_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_Response_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_WriteSettingsReq_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_SettingsResponse_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_PCoefficientsSet_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_PCoefficientsGet_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_T5CoefficientsSet_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_T5CoefficientsGet_msg: pb_msgdesc_t;
}
