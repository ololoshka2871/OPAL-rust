pub struct OutputStorage {
    pub targets: [u32; 2],
    pub results: [Option<u32>; 2],
    pub frequencys: [Option<f64>; 2],
    pub values: [Option<f64>; 2],

    pub t_cpu: f32,
    pub t_cpu_adc: u16,

    pub vbat_mv: u32,
    pub vbat_mv_adc: u16,
}

impl Default for OutputStorage {
    fn default() -> Self {
        Self {
            targets: [crate::config::INITIAL_FREQMETER_TARGET; 2],
            results: [None; 2],
            frequencys: [None; 2],
            values: [None; 2],

            t_cpu: 0.0,
            t_cpu_adc: 0,
            vbat_mv: 0,
            vbat_mv_adc: 0,
        }
    }
}
