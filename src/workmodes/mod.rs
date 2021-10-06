use freertos_rust::FreeRtosError;

pub mod high_performance_mode;
pub mod power_save_mode;

pub trait WorkMode {
    fn configure_clock(&mut self);
    fn start_threads(self) -> Result<(), FreeRtosError>;
}
