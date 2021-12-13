use super::TimerEvent;

pub trait FChProcessor {
    fn enable(&mut self);
    fn diasble(&mut self);

    fn target(&self) -> u32;

    fn restart(&mut self);
    fn reset_guard(&mut self);

    fn set_target(&mut self, new_target: u32, guard_ticks: u32);

    fn input_captured(&mut self, event: TimerEvent, captured: u32) -> Option<u32>;
}
