pub mod freqmeter;

pub trait Enable {
    fn start(&mut self);
    fn stop(&mut self) -> bool;
}
