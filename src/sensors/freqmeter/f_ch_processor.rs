use super::TimerEvent;

pub trait FChProcessor {
    fn enable(&mut self);
    fn diasbe(&mut self);

    fn restart(&mut self);

    fn set_target(&mut self, new_target: u32);

    fn input_captured(&mut self, event: TimerEvent, captured: u32) -> Option<u32>;
}
