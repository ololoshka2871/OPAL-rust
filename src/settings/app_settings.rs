#![allow(non_snake_case)]

use serde::Serialize;

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct P16Coeffs {
    pub Fp0: f32,
    pub Ft0: f32,
    pub A: [f32; crate::protobuf::P_COEFFS_COUNT],
}

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct T5Coeffs {
    pub F0: f32,
    pub T0: f32,
    pub C: [f32; crate::protobuf::T_COEFFS_COUNT],
}

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct WorkRange {
    pub minimum: f32,
    pub maximum: f32,
    pub absolute_maximum: f32,
}

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct CalibrationDate {
    pub Day: u32,
    pub Month: u32,
    pub Year: u32,
}

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct WriteConfig {
    pub BaseInterval_ms: u32,
    pub PWriteDevider: u32,
    pub TWriteDevider: u32,
}

#[repr(packed(1))]
#[derive(Debug, Copy, Clone, Serialize, Default)]
pub(crate) struct Monitoring {
    pub Ovarpress: bool,
    pub Ovarheat: bool,
    pub CPUOvarheat: bool,
    pub OverPower: bool,
}

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct AppSettings {
    pub Serial: u32,
    pub PMesureTime_ms: u32,
    pub TMesureTime_ms: u32,

    pub Fref: u32,

    pub P_enabled: bool,
    pub T_enabled: bool,
    pub TCPUEnabled: bool,
    pub VBatEnable: bool,

    pub P_Coefficients: P16Coeffs,
    pub T_Coefficients: T5Coeffs,

    pub PWorkRange: WorkRange,
    pub TWorkRange: WorkRange,
    pub TCPUWorkRange: WorkRange,
    pub VbatWorkRange: WorkRange,

    pub PZeroCorrection: f32,
    pub TZeroCorrection: f32,

    pub calibration_date: CalibrationDate,

    pub writeConfig: WriteConfig,

    pub startDelay: u32,

    pub password: [u8; crate::protobuf::PASSWORD_SIZE],

    pub monitoring: Monitoring,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct NonStoreSettings {
    pub current_password: [u8; 10],
}

impl Monitoring {
    pub fn is_set(&self) -> bool {
        self.Ovarpress | self.Ovarheat | self.CPUOvarheat | self.OverPower
    }
}
