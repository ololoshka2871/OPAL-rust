#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(array_zip)]
#![feature(core_intrinsics)]
#![feature(macro_metavar_expr)]

extern crate alloc;

// экономим место
use panic_halt as _;

mod control;
mod gcode;
mod support;
mod threads;
mod time_base;
mod workmodes;

pub mod config;
pub mod config_pins;

#[cfg(debug_assertions)]
mod master_value_stat;

use cortex_m_rt::entry;

use stm32f1xx_hal::stm32;

use crate::workmodes::{high_performance_mode::HighPerformanceMode, WorkMode};

//---------------------------------------------------------------

#[global_allocator]
static GLOBAL: freertos_rust::FreeRtosAllocator = freertos_rust::FreeRtosAllocator;

//---------------------------------------------------------------

#[entry]
fn main() -> ! {
    /*
    #[cfg(debug_assertions)]
    cortex_m::asm::bkpt();
    */

    defmt::trace!("++ Start up! ++");

    let p = unsafe { cortex_m::Peripherals::take().unwrap_unchecked() };
    let dp = unsafe { stm32::Peripherals::take().unwrap_unchecked() };

    start_at_mode::<HighPerformanceMode>(p, dp).expect("expect1");

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

    #[cfg(debug_assertions)]
    master_value_stat::init_master_getter(time_base::master_counter::MasterCounter::acquire());

    mode.start_threads()
}

//-----------------------------------------------------------------------------
