#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(int_abs_diff)]

extern crate alloc;

mod settings;
mod support;
mod threads;
mod workmodes;
mod control;

pub mod config;
pub mod config_pins;

use cortex_m_rt::entry;

use stm32l4xx_hal::stm32;

use crate::{
    support::free_rtos_error_ext::FreeRtosErrorContainer,
    workmodes::{high_performance_mode::HighPerformanceMode, WorkMode},
};

//---------------------------------------------------------------

#[global_allocator]
static GLOBAL: freertos_rust::FreeRtosAllocator = freertos_rust::FreeRtosAllocator;

//---------------------------------------------------------------

#[entry]
fn main() -> ! {
    // #[cfg(debug_assertions)]
    // cortex_m::asm::bkpt();

    defmt::trace!("++ Start up! ++");

    let p = unsafe { cortex_m::Peripherals::take().unwrap_unchecked() };
    let dp = unsafe { stm32::Peripherals::take().unwrap_unchecked() };

    start_at_mode::<HighPerformanceMode>(p, dp)
        .unwrap_or_else(|e| defmt::panic!("Failed to start thread: {}", FreeRtosErrorContainer(e)));

    freertos_rust::FreeRtosUtils::start_scheduler();
}

fn start_at_mode<T>(
    p: cortex_m::Peripherals,
    dp: stm32::Peripherals,
) -> Result<(), freertos_rust::FreeRtosError>
where
    T: WorkMode<T>,
{
    let mut mode = T::new(p, dp);
    mode.ini_static();
    mode.configure_clock();
    mode.print_clock_config();
    mode.start_threads()
}

//-----------------------------------------------------------------------------
