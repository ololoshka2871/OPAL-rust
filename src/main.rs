#![no_std]
#![no_main]

// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod support;
mod workmodes;

use cortex_m_rt::entry;

use stm32l4xx_hal::stm32;
use support::{usb_connection_checker::UsbConnectionChecker, vusb_monitor::VUsbMonitor};

use crate::{
    support::free_rtos_error_ext::FreeRtosErrorContainer,
    workmodes::{
        high_performance_mode::HighPerformanceMode, power_save_mode::PowerSaveMode, WorkMode,
    },
};

mod threads;

//---------------------------------------------------------------

#[global_allocator]
static GLOBAL: freertos_rust::FreeRtosAllocator = freertos_rust::FreeRtosAllocator;

//---------------------------------------------------------------

#[entry]
fn main() -> ! {
    defmt::trace!("-- Start up! --");

    let dp = stm32::Peripherals::take().unwrap();

    let start_res = if is_usb_connected() {
        defmt::info!("USB connected, CPU max performance mode");
        let mut mode = HighPerformanceMode::new(dp);
        mode.configure_clock();
        mode.start_threads()
    } else {
        defmt::info!("USB not connected, self-writer mode");
        let mut mode = PowerSaveMode::new(dp);
        mode.configure_clock();
        mode.start_threads()
    };
    
    start_res.unwrap_or_else(|e| {
        defmt::panic!("Failed to start thread: {}", FreeRtosErrorContainer(e))
    }); 

    freertos_rust::FreeRtosUtils::start_scheduler();
}

fn is_usb_connected() -> bool {
    let rcc = unsafe { &*stm32::RCC::ptr() };
    let pwr = unsafe { &*stm32::PWR::ptr() };

    VUsbMonitor::new(rcc, pwr).is_usb_connected()
}