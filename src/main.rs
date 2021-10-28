#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod protobuf;
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

    test_nanopb();

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
    mode.configure_clock();
    mode.print_clock_config();
    mode.start_threads()
}

fn is_usb_connected() -> bool {
    let rcc = unsafe { &*stm32::RCC::ptr() };
    let pwr = unsafe { &*stm32::PWR::ptr() };

    VUsbMonitor::new(rcc, pwr).is_usb_connected()
}

fn test_nanopb() {
    use nanopb_rs::{IStream, OStream};

    let mut buf = [0_u8; 256];
    let len: usize;
    {
        let mut msg = protobuf::_ru_sktbelpa_pressure_self_writer_Request::default();
        msg.id = 42;
        let mut os = OStream::from_buffer(&mut buf);

        let _ = os
            .encode(
                protobuf::_ru_sktbelpa_pressure_self_writer_Request::fields(),
                &msg,
            )
            .map_err(|e| panic!("{}", e));

        len = os.bytes_writen();
    }
    {
        let mut is = IStream::from_buffer(&buf[..len]);

        let msg = is
            .encode::<protobuf::_ru_sktbelpa_pressure_self_writer_Request>(
                protobuf::_ru_sktbelpa_pressure_self_writer_Request::fields(),
            )
            .unwrap_or_else(|e| panic!("{}", e));
        assert_eq!(msg.id, 42);
    }
}
