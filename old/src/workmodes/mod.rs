use alloc::sync::Arc;
use freertos_rust::{FreeRtosError, Mutex};

pub mod high_performance_mode;

pub(crate) mod common;

pub trait WorkMode<T> {
    fn new(p: cortex_m::Peripherals, dp: stm32f1xx_hal::device::Peripherals) -> T;
    fn ini_static(&mut self);
    fn configure_clock(&mut self);
    fn start_threads(self) -> Result<(), FreeRtosError>;
    fn print_clock_config(&self);
    fn flash(&mut self) -> Arc<Mutex<stm32f1xx_hal::flash::Parts>>;
    fn crc(&mut self) -> Arc<Mutex<stm32f1xx_hal::crc::Crc>>;
}

fn configure_crc_module(config: stm32f1xx_hal::crc::Crc) -> stm32f1xx_hal::crc::Crc {
    config
}
