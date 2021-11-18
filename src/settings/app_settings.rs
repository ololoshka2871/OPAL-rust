#![allow(non_snake_case)]

use serde::Serialize;
use stm32l4xx_hal::device::flash::cr;

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct P16Coeffs {
    pub Fp0: f32,
    pub Ft0: f32,
    pub A: [f32; 16],
}

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct T5Coeffs {
    pub F0: f32,
    pub T0: f32,
    pub C: [f32; 5],
}

#[derive(Debug, Copy, Clone, Serialize)]
pub(crate) struct AppSettings {
    pub Serial: u32,
    pub PMesureTime_ms: u32,
    pub TMesureTime_ms: u32,

    pub Fref: u32,

    pub P_enabled: bool,
    pub T_enabled: bool,

    pub P_Coefficients: P16Coeffs,
    pub T_Coefficients: T5Coeffs,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct NonStoreSettings {
    pub current_password: [u8; 10]
}