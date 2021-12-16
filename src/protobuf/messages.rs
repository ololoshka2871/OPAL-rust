#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(
    env!("OUT_DIR"),
    "/ru.sktbelpa.pressure_self_writer.rs"
));

pub use nanopb_rs::pb_callback_t;
use nanopb_rs::pb_msgdesc_t;
pub use nanopb_rs::pb_size_t;

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

pub const _ru_sktbelpa_pressure_self_writer_FlashStatus_Status_ru_sktbelpa_pressure_self_writer_FlashStatus_Status_OK : _ru_sktbelpa_pressure_self_writer_FlashStatus_Status = 0 ;
pub const _ru_sktbelpa_pressure_self_writer_FlashStatus_Status_ru_sktbelpa_pressure_self_writer_FlashStatus_Status_Ereasing : _ru_sktbelpa_pressure_self_writer_FlashStatus_Status = 1 ;
pub const _ru_sktbelpa_pressure_self_writer_FlashStatus_Status_ru_sktbelpa_pressure_self_writer_FlashStatus_Status_ResetMonitoringFailed : _ru_sktbelpa_pressure_self_writer_FlashStatus_Status = 2 ;
pub type _ru_sktbelpa_pressure_self_writer_FlashStatus_Status = u32;
pub use self::_ru_sktbelpa_pressure_self_writer_FlashStatus_Status as ru_sktbelpa_pressure_self_writer_FlashStatus_Status;

//----------------------------------------------------------------------------------------------------

pub const _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_Pa : _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits = 2228224 ;
pub const _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_Bar : _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits = 5111808 ;
pub const _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_At : _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits = 10551296 ;
pub const _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_mmH20 : _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits = 10616832 ;
pub const _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_mHg : _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits = 10682368 ;
pub const _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_Atm : _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits = 10747904 ;
pub const _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_ru_sktbelpa_pressure_self_writer_PressureMeassureUnits_PSI : _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits = 11206656 ;
pub type _ru_sktbelpa_pressure_self_writer_PressureMeassureUnits = u32;
pub use self::_ru_sktbelpa_pressure_self_writer_PressureMeassureUnits as ru_sktbelpa_pressure_self_writer_PressureMeassureUnits;

//----------------------------------------------------------------------------------------------------

pub const P_COEFFS_COUNT: usize = 16;
pub const T_COEFFS_COUNT: usize = 5;
pub const PASSWORD_SIZE: usize = 10;

#[repr(C)]
#[derive(Debug, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_PCoefficients {
    pub has_Fp0: bool,
    pub Fp0: f32,
    pub has_Ft0: bool,
    pub Ft0: f32,
    pub has_A0: bool,
    pub A0: f32,
    pub has_A1: bool,
    pub A1: f32,
    pub has_A2: bool,
    pub A2: f32,
    pub has_A3: bool,
    pub A3: f32,
    pub has_A4: bool,
    pub A4: f32,
    pub has_A5: bool,
    pub A5: f32,
    pub has_A6: bool,
    pub A6: f32,
    pub has_A7: bool,
    pub A7: f32,
    pub has_A8: bool,
    pub A8: f32,
    pub has_A9: bool,
    pub A9: f32,
    pub has_A10: bool,
    pub A10: f32,
    pub has_A11: bool,
    pub A11: f32,
    pub has_A12: bool,
    pub A12: f32,
    pub has_A13: bool,
    pub A13: f32,
    pub has_A14: bool,
    pub A14: f32,
    pub has_A15: bool,
    pub A15: f32,
}
pub type ru_sktbelpa_pressure_self_writer_PCoefficients =
    _ru_sktbelpa_pressure_self_writer_PCoefficients;

#[repr(C)]
#[derive(Debug, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_T5Coefficients {
    pub has_T0: bool,
    pub T0: f32,
    pub has_F0: bool,
    pub F0: f32,
    pub has_C1: bool,
    pub C1: f32,
    pub has_C2: bool,
    pub C2: f32,
    pub has_C3: bool,
    pub C3: f32,
    pub has_C4: bool,
    pub C4: f32,
    pub has_C5: bool,
    pub C5: f32,
}
pub type ru_sktbelpa_pressure_self_writer_T5Coefficients =
    _ru_sktbelpa_pressure_self_writer_T5Coefficients;

#[repr(C)]
#[derive(Debug, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_SettingsResponse {
    pub Serial: u32,
    pub PMesureTime_ms: u32,
    pub TMesureTime_ms: u32,
    pub Fref: u32,
    pub PEnabled: bool,
    pub TEnabled: bool,
    pub TCPUEnabled: bool,
    pub VBatEnable: bool,
    pub PCoefficients: ru_sktbelpa_pressure_self_writer_PCoefficients,
    pub TCoefficients: ru_sktbelpa_pressure_self_writer_T5Coefficients,
    pub PWorkRange: ru_sktbelpa_pressure_self_writer_WorkRange,
    pub TWorkRange: ru_sktbelpa_pressure_self_writer_WorkRange,
    pub TCPUWorkRange: ru_sktbelpa_pressure_self_writer_WorkRange,
    pub BatWorkRange: ru_sktbelpa_pressure_self_writer_WorkRange,
    pub CalibrationDate: ru_sktbelpa_pressure_self_writer_CalibrationDate,
    pub PZeroCorrection: f32,
    pub TZeroCorrection: f32,
    pub writeConfig: ru_sktbelpa_pressure_self_writer_WriteConfig,
    pub startDelay: u32,
    pub pressureMeassureUnits: ru_sktbelpa_pressure_self_writer_PressureMeassureUnits,
    pub password: [u8; PASSWORD_SIZE + 1],
}
pub type ru_sktbelpa_pressure_self_writer_SettingsResponse =
    _ru_sktbelpa_pressure_self_writer_SettingsResponse;

#[repr(C)]
#[derive(Debug)]
pub struct _ru_sktbelpa_pressure_self_writer_WriteSettingsReq {
    pub has_setSerial: bool,
    pub setSerial: u32,
    pub has_setPMesureTime_ms: bool,
    pub setPMesureTime_ms: u32,
    pub has_setTMesureTime_ms: bool,
    pub setTMesureTime_ms: u32,
    pub has_setFref: bool,
    pub setFref: u32,
    pub has_setPEnabled: bool,
    pub setPEnabled: bool,
    pub has_setTEnabled: bool,
    pub setTEnabled: bool,
    pub has_setTCPUEnabled: bool,
    pub setTCPUEnabled: bool,
    pub has_setVBatEnable: bool,
    pub setVBatEnable: bool,
    pub has_setPCoefficients: bool,
    pub setPCoefficients: ru_sktbelpa_pressure_self_writer_PCoefficients,
    pub has_setTCoefficients: bool,
    pub setTCoefficients: ru_sktbelpa_pressure_self_writer_T5Coefficients,
    pub has_setPWorkRange: bool,
    pub setPWorkRange: ru_sktbelpa_pressure_self_writer_WorkRange,
    pub has_setTWorkRange: bool,
    pub setTWorkRange: ru_sktbelpa_pressure_self_writer_WorkRange,
    pub has_setTCPUWorkRange: bool,
    pub setTCPUWorkRange: ru_sktbelpa_pressure_self_writer_WorkRange,
    pub has_setBatWorkRange: bool,
    pub setBatWorkRange: ru_sktbelpa_pressure_self_writer_WorkRange,
    pub has_setCalibrationDate: bool,
    pub setCalibrationDate: ru_sktbelpa_pressure_self_writer_CalibrationDate,
    pub has_setPZeroCorrection: bool,
    pub setPZeroCorrection: f32,
    pub has_setTZeroCorrection: bool,
    pub setTZeroCorrection: f32,
    pub has_setWriteConfig: bool,
    pub setWriteConfig: ru_sktbelpa_pressure_self_writer_WriteConfig,
    pub has_setStartDelay: bool,
    pub setStartDelay: u32,
    pub has_setPressureMeassureUnits: bool,
    pub setPressureMeassureUnits: ru_sktbelpa_pressure_self_writer_PressureMeassureUnits,
    pub has_setPassword: bool,
    pub setPassword: [u8; PASSWORD_SIZE + 1],
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
    pub has_getInfo: bool,
    pub getInfo: ru_sktbelpa_pressure_self_writer_Empty,
    pub has_getOutputValues: bool,
    pub getOutputValues: ru_sktbelpa_pressure_self_writer_OutputReq,
    pub has_flashCommand: bool,
    pub flashCommand: ru_sktbelpa_pressure_self_writer_FlasCommand,
    pub has_changePassword: bool,
    pub changePassword: ru_sktbelpa_pressure_self_writer_ChangePassword,
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
    pub has_info: bool,
    pub info: ru_sktbelpa_pressure_self_writer_InfoResponse,
    pub has_output: bool,
    pub output: ru_sktbelpa_pressure_self_writer_OutputResponse,
    pub has_flashStatus: bool,
    pub flashStatus: ru_sktbelpa_pressure_self_writer_FlashStatus,
    pub has_changePasswordStatus: bool,
    pub changePasswordStatus: ru_sktbelpa_pressure_self_writer_ChangePasswordStatus,
}
pub type ru_sktbelpa_pressure_self_writer_Response = _ru_sktbelpa_pressure_self_writer_Response;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _ru_sktbelpa_pressure_self_writer_Empty {
    pub dummy_field: u8,
}
pub type ru_sktbelpa_pressure_self_writer_Empty = _ru_sktbelpa_pressure_self_writer_Empty;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _ru_sktbelpa_pressure_self_writer_OutputReq {
    pub has_getMainValues: bool,
    pub getMainValues: ru_sktbelpa_pressure_self_writer_Empty,
    pub has_getF: bool,
    pub getF: ru_sktbelpa_pressure_self_writer_Empty,
    pub has_getRAW: bool,
    pub getRAW: ru_sktbelpa_pressure_self_writer_Empty,
}
pub type ru_sktbelpa_pressure_self_writer_OutputReq = _ru_sktbelpa_pressure_self_writer_OutputReq;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _ru_sktbelpa_pressure_self_writer_FlasCommand {
    pub has_ResetMonitoring: bool,
    pub ResetMonitoring: ru_sktbelpa_pressure_self_writer_Empty,
    pub has_ClearMemory: bool,
    pub ClearMemory: bool,
}
pub type ru_sktbelpa_pressure_self_writer_FlasCommand =
    _ru_sktbelpa_pressure_self_writer_FlasCommand;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _ru_sktbelpa_pressure_self_writer_ChangePassword {
    pub newPassword: [u8; PASSWORD_SIZE + 1],
}
pub type ru_sktbelpa_pressure_self_writer_ChangePassword =
    _ru_sktbelpa_pressure_self_writer_ChangePassword;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_InfoResponse {
    pub HW_Version: u32,
    pub SW_Version: u64,
    pub PressureChannelFailed: bool,
    pub TemperatureChannelFailed: bool,
    pub PressureOutOfrange: bool,
    pub TemperatureOutOfrange: bool,
    pub CPUTemperatureOutOfrange: bool,
    pub VbatOutOfrange: bool,
    pub OverpressDetected: bool,
    pub OverheatDetected: bool,
    pub OverheatCPUDetected: bool,
    pub OverVbatDetected: bool,
}
pub type ru_sktbelpa_pressure_self_writer_InfoResponse =
    _ru_sktbelpa_pressure_self_writer_InfoResponse;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_OutputResponse {
    pub has_pressure: bool,
    pub pressure: f32,
    pub has_temperature: bool,
    pub temperature: f32,
    pub has_TCPU: bool,
    pub TCPU: f32,
    pub has_Vbat: bool,
    pub Vbat: f32,
    pub has_FP: bool,
    pub FP: f32,
    pub has_FT: bool,
    pub FT: f32,
    pub has_P_result: bool,
    pub P_result: ru_sktbelpa_pressure_self_writer_FreqmeterResult,
    pub has_T_result: bool,
    pub T_result: ru_sktbelpa_pressure_self_writer_FreqmeterResult,
    pub has_ADC_TCPU: bool,
    pub ADC_TCPU: u32,
    pub has_ADC_Vbat: bool,
    pub ADC_Vbat: u32,
}
pub type ru_sktbelpa_pressure_self_writer_OutputResponse =
    _ru_sktbelpa_pressure_self_writer_OutputResponse;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_FlashStatus {
    pub FlashPageSize: u32,
    pub FlashPages: u32,
    pub FlashUsedPages: u32,
    pub status: ru_sktbelpa_pressure_self_writer_FlashStatus_Status,
}
pub type ru_sktbelpa_pressure_self_writer_FlashStatus =
    _ru_sktbelpa_pressure_self_writer_FlashStatus;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_ChangePasswordStatus {
    pub passwordChanged: bool,
}
pub type ru_sktbelpa_pressure_self_writer_ChangePasswordStatus =
    _ru_sktbelpa_pressure_self_writer_ChangePasswordStatus;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_FreqmeterResult {
    pub Target: u32,
    pub Result: u32,
}
pub type ru_sktbelpa_pressure_self_writer_FreqmeterResult =
    _ru_sktbelpa_pressure_self_writer_FreqmeterResult;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_WorkRange {
    pub has_minimum: bool,
    pub minimum: f32,
    pub has_maximum: bool,
    pub maximum: f32,
    pub has_absolute_maximum: bool,
    pub absolute_maximum: f32,
}
pub type ru_sktbelpa_pressure_self_writer_WorkRange = _ru_sktbelpa_pressure_self_writer_WorkRange;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_CalibrationDate {
    pub has_Day: bool,
    pub Day: u32,
    pub has_Month: bool,
    pub Month: u32,
    pub has_Year: bool,
    pub Year: u32,
}
pub type ru_sktbelpa_pressure_self_writer_CalibrationDate =
    _ru_sktbelpa_pressure_self_writer_CalibrationDate;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct _ru_sktbelpa_pressure_self_writer_WriteConfig {
    pub has_BaseInterval_ms: bool,
    pub BaseInterval_ms: u32,
    pub has_PWriteDevider: bool,
    pub PWriteDevider: u32,
    pub has_TWriteDevider: bool,
    pub TWriteDevider: u32,
}
pub type ru_sktbelpa_pressure_self_writer_WriteConfig =
    _ru_sktbelpa_pressure_self_writer_WriteConfig;

extern "C" {
    pub static ru_sktbelpa_pressure_self_writer_Request_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_Response_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_WriteSettingsReq_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_SettingsResponse_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_WorkRange_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_CalibrationDate_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_WriteConfig_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_PCoefficients_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_T5Coefficients_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_InfoResponse_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_OutputReq_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_OutputResponse_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_FreqmeterResult_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_FlasCommand_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_FlashStatus_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_ChangePassword_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_ChangePasswordStatus_msg: pb_msgdesc_t;
    pub static ru_sktbelpa_pressure_self_writer_Empty_msg: pb_msgdesc_t;
}

impl ru_sktbelpa_pressure_self_writer_PCoefficients {
    pub(crate) fn from(p_coeffs: &crate::settings::app_settings::P16Coeffs) -> Self {
        Self {
            has_Fp0: true,
            Fp0: p_coeffs.Fp0,
            has_Ft0: true,
            Ft0: p_coeffs.Ft0,

            has_A0: true,
            A0: p_coeffs.A[0],
            has_A1: true,
            A1: p_coeffs.A[1],
            has_A2: true,
            A2: p_coeffs.A[2],
            has_A3: true,
            A3: p_coeffs.A[3],
            has_A4: true,
            A4: p_coeffs.A[4],
            has_A5: true,
            A5: p_coeffs.A[5],
            has_A6: true,
            A6: p_coeffs.A[6],
            has_A7: true,
            A7: p_coeffs.A[7],
            has_A8: true,
            A8: p_coeffs.A[8],
            has_A9: true,
            A9: p_coeffs.A[9],
            has_A10: true,
            A10: p_coeffs.A[10],
            has_A11: true,
            A11: p_coeffs.A[11],
            has_A12: true,
            A12: p_coeffs.A[12],
            has_A13: true,
            A13: p_coeffs.A[13],
            has_A14: true,
            A14: p_coeffs.A[14],
            has_A15: true,
            A15: p_coeffs.A[15],
        }
    }
}

impl ru_sktbelpa_pressure_self_writer_T5Coefficients {
    pub(crate) fn from(t_coeffs: &crate::settings::app_settings::T5Coeffs) -> Self {
        Self {
            has_T0: true,
            T0: t_coeffs.T0,
            has_F0: true,
            F0: t_coeffs.F0,

            has_C1: true,
            C1: t_coeffs.C[0],
            has_C2: true,
            C2: t_coeffs.C[1],
            has_C3: true,
            C3: t_coeffs.C[2],
            has_C4: true,
            C4: t_coeffs.C[3],
            has_C5: true,
            C5: t_coeffs.C[4],
        }
    }
}

impl ru_sktbelpa_pressure_self_writer_WorkRange {
    pub(crate) fn from(wr: &crate::settings::app_settings::WorkRange) -> Self {
        Self {
            has_minimum: true,
            minimum: wr.minimum,
            has_maximum: true,
            maximum: wr.maximum,
            has_absolute_maximum: true,
            absolute_maximum: wr.absolute_maximum,
        }
    }
}

#[derive(Debug)]
pub enum DateField {
    Day,
    Month,
    Past,
}

impl ru_sktbelpa_pressure_self_writer_CalibrationDate {
    pub(crate) fn from(cd: &crate::settings::app_settings::CalibrationDate) -> Self {
        Self {
            has_Day: true,
            Day: cd.Day,
            has_Month: true,
            Month: cd.Month,
            has_Year: true,
            Year: cd.Year,
        }
    }

    pub fn validate(&self) -> Result<(), DateField> {
        use my_proc_macro::{build_day, build_month, build_year};

        if self.has_Day && self.Day > 31 {
            return Err(DateField::Day);
        }
        if self.has_Month && (self.Month > 12 || self.Month < 1) {
            return Err(DateField::Month);
        }
        if self.has_Year {
            if self.Year < build_year!() {
                return Err(DateField::Past);
            }
            if self.has_Day && self.has_Month {
                if self.Month < build_month!() {
                    return Err(DateField::Past);
                } else if self.Month == build_month!() && self.Day < build_day!() {
                    return Err(DateField::Past);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum WorkRangeError {
    MinimumAboveMaximum,
    MinimumAboveAbsoluteMaximum,
    MaximumAboveAbasoluteMaximum,
}

impl ru_sktbelpa_pressure_self_writer_WorkRange {
    pub fn validate(&self) -> Result<(), WorkRangeError> {
        if self.has_absolute_maximum {
            if self.has_maximum && self.absolute_maximum < self.maximum {
                return Err(WorkRangeError::MaximumAboveAbasoluteMaximum);
            }
            if self.has_minimum && self.absolute_maximum < self.minimum {
                return Err(WorkRangeError::MinimumAboveMaximum);
            }
        }

        if self.has_maximum && self.has_minimum && self.maximum < self.minimum {
            return Err(WorkRangeError::MinimumAboveMaximum);
        }

        Ok(())
    }
}

impl ru_sktbelpa_pressure_self_writer_WriteConfig {
    pub(crate) fn from(wc: &crate::settings::app_settings::WriteConfig) -> Self {
        Self {
            has_BaseInterval_ms: true,
            BaseInterval_ms: wc.BaseInterval_ms,
            has_PWriteDevider: true,
            PWriteDevider: wc.PWriteDevider,
            has_TWriteDevider: true,
            TWriteDevider: wc.TWriteDevider,
        }
    }
}
