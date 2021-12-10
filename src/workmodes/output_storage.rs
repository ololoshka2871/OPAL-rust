pub struct OutputStorage {
    pub targets: [u32; 2],
    pub results: [Option<u32>; 2],
    pub frequencys: [Option<f64>; 2],
    pub values: [Option<f64>; 2],
}

impl Default for OutputStorage {
    fn default() -> Self {
        Self {
            targets: [crate::config::INITIAL_FREQMETER_TARGET; 2],
            results: [None; 2],
            frequencys: [None; 2],
            values: [None; 2],
        }
    }
}
