#![allow(non_snake_case)]
#![allow(dead_code)] // FIXME

#[derive(Debug, Copy, Clone)]
pub(crate) struct P16Coeffs {
    pub Fp0: f32,
    pub Ft0: f32,
    pub A: [f32; 16],
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct T5Coeffs {
    pub F0: f32,
    pub T0: f32,
    pub C: [f32; 5],
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct AppSettings {
    pub serial: u32,
    pub pmesure_time_ms: u32,
    pub tmesure_time_ms: u32,

    pub fref: u32,

    pub p_enabled: bool,
    pub t_enabled: bool,

    pub pcoefficients: P16Coeffs,
    pub tcoefficients: T5Coeffs,
}
