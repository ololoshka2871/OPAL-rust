#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]

extern crate alloc;

mod main_data_storage;
mod protobuf;
mod sensors;
mod settings;
mod support;
mod workmodes;

pub mod config;

#[cfg(debug_assertions)]
mod master_value_stat;

use cortex_m_rt::entry;

use stm32l4xx_hal::stm32;
use support::{usb_connection_checker::UsbConnectionChecker, vusb_monitor::VUsbMonitor};

use crate::{
    support::free_rtos_error_ext::FreeRtosErrorContainer,
    workmodes::{
        high_performance_mode::HighPerformanceMode, recorder_mode::RecorderMode, WorkMode,
    },
};

mod threads;

//---------------------------------------------------------------

#[global_allocator]
static GLOBAL: freertos_rust::FreeRtosAllocator = freertos_rust::FreeRtosAllocator;

//---------------------------------------------------------------

#[entry]
fn main() -> ! {
    defmt::trace!("++ Start up! ++");

    let p = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    let start_res = if is_usb_connected() {
        defmt::info!("USB connected, CPU max performance mode");
        start_at_mode::<HighPerformanceMode>(p, dp)
    } else {
        defmt::info!("USB not connected, self-writer mode");
        start_at_mode::<RecorderMode>(p, dp)
    };

    start_res
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

    #[cfg(debug_assertions)]
    master_value_stat::init_master_getter(
        sensors::freqmeter::master_counter::MasterCounter::allocate().unwrap(),
    );

    mode.start_threads()
}

fn is_usb_connected() -> bool {
    let rcc = unsafe { &*stm32::RCC::ptr() };
    let pwr = unsafe { &*stm32::PWR::ptr() };

    VUsbMonitor::new(rcc, pwr).is_usb_connected()
}

//-----------------------------------------------------------------------------
