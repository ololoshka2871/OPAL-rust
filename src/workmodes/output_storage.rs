pub struct OutputStorage {
    pub frequencys: [f64; 2],
    pub targets: [u32; 2],
    pub results: [Option<u32>; 2],
}

impl Default for OutputStorage {
    fn default() -> Self {
        Self {
            frequencys: [f64::NAN; 2],
            targets: [crate::config::INITIAL_FREQMETER_TARGET; 2],
            results: [None; 2],
        }
    }
}
