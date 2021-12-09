use alloc::sync::Arc;
use freertos_rust::{FreeRtosError, Mutex};

pub mod high_performance_mode;
//pub mod high_performance_processor;
pub mod power_save_mode;

pub(crate) mod common;
//mod my_clock_freeze;

pub mod output_storage;
//pub mod processing;

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
    // FIXME
    //const CRC_POLY: u32 = 0xffff_ffff;
    //const CRC_INITIAL: u32 = 0xffff_ffff;

    config
        //.polynomial(stm32l4xx_hal::crc::Polynomial::L32(CRC_POLY))
        //.initial_value(CRC_INITIAL)
        .freeze()
}
