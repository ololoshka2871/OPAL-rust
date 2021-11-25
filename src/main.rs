#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod main_data_storage;
mod protobuf;
mod sensors;
mod settings;
mod support;
mod workmodes;

pub mod config;

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
    defmt::trace!("++ Start up! ++");

    let p = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    let start_res = if is_usb_connected() {
        defmt::info!("USB connected, CPU max performance mode");
        start_at_mode::<HighPerformanceMode>(p, dp)
    } else {
        defmt::info!("USB not connected, self-writer mode");
        start_at_mode::<PowerSaveMode>(p, dp)
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
    //use stm32l4xx_hal::time::U32Ext;

    let mut mode = T::new(p, dp);
    mode.ini_static();
    mode.configure_clock();
    mode.print_clock_config();
    mode.start_threads()

    /*
    let mut flash = stm32l4xx_hal::prelude::_stm32l4_hal_FlashExt::constrain(dp.FLASH);
    let mut rcc = stm32l4xx_hal::rcc::RccExt::constrain(dp.RCC);
    let mut pwr = stm32l4xx_hal::pwr::PwrExt::constrain(dp.PWR, &mut rcc.apb1r1);

    // Try a different clock configuration
    let clocks = rcc.cfgr.freeze(&mut flash.acr, &mut pwr);

    let _timer = stm32l4xx_hal::timer::Timer::tim6(dp.TIM6, 1000.hz(), clocks, &mut rcc.apb1r1);

    loop {}
    */
}

fn is_usb_connected() -> bool {
    let rcc = unsafe { &*stm32::RCC::ptr() };
    let pwr = unsafe { &*stm32::PWR::ptr() };

    VUsbMonitor::new(rcc, pwr).is_usb_connected()
}
