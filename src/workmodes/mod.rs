use freertos_rust::FreeRtosError;

pub mod high_performance_mode;
pub mod power_save_mode;

mod common;
//mod my_clock_freeze;

pub trait WorkMode<T> {
    fn new(p: cortex_m::Peripherals, dp: stm32l4xx_hal::device::Peripherals) -> T;
    fn configure_clock(&mut self);
    fn start_threads(self) -> Result<(), FreeRtosError>;
    fn print_clock_config(&self);
}
