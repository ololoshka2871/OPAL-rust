use alloc::sync::Arc;
use freertos_rust::{FreeRtosError, Mutex};

pub mod high_performance_mode;

pub(crate) mod common;

pub trait WorkMode<T> {
    fn new(p: cortex_m::Peripherals, dp: stm32l4xx_hal::device::Peripherals) -> T;
    fn ini_static(&mut self);
    fn configure_clock(&mut self);
    fn start_threads(self) -> Result<(), FreeRtosError>;
    fn print_clock_config(&self);
    fn flash(&mut self) -> Arc<Mutex<stm32l4xx_hal::flash::Parts>>;
    fn crc(&mut self) -> Arc<Mutex<stm32l4xx_hal::crc::Crc>>;
}

fn configure_crc_module(config: stm32l4xx_hal::crc::Config) -> stm32l4xx_hal::crc::Crc {
    config
        // теперь результат соответсвует zlib овскому, но !нужно инвертировать!
        // https://stackoverflow.com/a/48883954
        .input_bit_reversal(stm32l4xx_hal::crc::BitReversal::ByByte)
        .output_bit_reversal(true)
        .freeze()
}
