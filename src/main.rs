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

use sensors::freqmeter::master_counter::{MasterCounter, MasterTimerInfo};
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

struct MasterGetter {
    master: MasterTimerInfo,
    val: u64,
}

impl MasterGetter {
    fn new(master: MasterTimerInfo) -> Self {
        Self { master, val: 0 }
    }

    fn value(&mut self) -> u32 {
        let v = self.master.value().0 as u64;
        if v < self.val & 0xffff_ffff {
            self.val = ((self.val >> 32) + 1) << 32;
        } else {
            self.val &= 0xffff_ffff_0000_0000;
        }
        self.val |= v as u64;

        (self.val >> 16) as u32
    }
}

static mut MASTER_TIMER_VALUE_GETTER: Option<MasterGetter> = None;

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
    let mut mode = T::new(p, dp);
    mode.ini_static();
    mode.configure_clock();
    mode.print_clock_config();

    {
        let mut master = MasterCounter::allocate().unwrap();
        master.want_start();
        unsafe {
            MASTER_TIMER_VALUE_GETTER = Some(MasterGetter::new(master));
        };
    }

    mode.start_threads()
}

fn is_usb_connected() -> bool {
    let rcc = unsafe { &*stm32::RCC::ptr() };
    let pwr = unsafe { &*stm32::PWR::ptr() };

    VUsbMonitor::new(rcc, pwr).is_usb_connected()
}

//-----------------------------------------------------------------------------

#[allow(non_camel_case_types)]
#[no_mangle]
pub unsafe extern "C" fn getMaterCounterValue() -> u32 {
    if MASTER_TIMER_VALUE_GETTER.is_some() {
        MASTER_TIMER_VALUE_GETTER.as_mut().unwrap().value()
    } else {
        0
    }
}
