pub trait FChProcessor<TSE> {
    fn enable(&mut self);
    fn diasbe(&mut self);

    fn is_initial_result(&mut self) -> bool;

    fn adaptate(&mut self) -> Result<u32, ()>;

    fn input_captured(&mut self, captured: u32) -> Option<u32>;
    fn calc_freq(&mut self, target: u32, result: u32) -> Result<f64, TSE>;
}
