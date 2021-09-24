#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

extern crate panic_halt; // panic handler

use core::alloc::Layout;
use cortex_m::asm;
use cortex_m_rt::exception;
use cortex_m_rt::{entry, ExceptionFrame};
use freertos_rust::*;

use cortex_m_log::printer::{semihosting, Printer};

use heatshrink_rust::decoder::HeatshrinkDecoder;
use heatshrink_rust::encoder::HeatshrinkEncoder;

use stm32l4xx_hal::{prelude::*, stm32};

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

#[entry]
fn main() -> ! {
    configure_clocks();

    Task::new()
        .name("thread")
        .stack_size(2548)
        .priority(TaskPriority(2))
        .start(move || {
            let mut shost = semihosting::InterruptOk::<_>::stdout().unwrap();

            let src = [0u8; 8];

            let mut it_src = src.iter().map(|a| *a);

            let mut enc = HeatshrinkEncoder::from_source(&mut it_src);
            let mut dec = HeatshrinkDecoder::from_source(&mut enc);
            loop {
                if let Some(b) = dec.next() {
                    shost.println(format_args!("=={:X}", b));
                } else {
                    break;
                }
            }

            loop {}
        })
        .unwrap();
    FreeRtosUtils::start_scheduler();
}

fn configure_clocks() {
    use stm32l4xx_hal::rcc::{PllConfig, PllDivider};

    if let Some(dp) = stm32::Peripherals::take() {
        let mut rcc = dp.RCC.constrain();

        let mut flash = dp.FLASH.constrain();
        let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);
        let clocks = rcc
            .cfgr
            //.sysclk(24.mhz())
            .hse(
                12.mhz(),
                stm32l4xx_hal::rcc::CrystalBypass::Disable,
                stm32l4xx_hal::rcc::ClockSecuritySystem::Enable,
            )
            .pll_source(stm32l4xx_hal::rcc::PllSource::HSE)
            .sysclk_with_pll(24.mhz(), PllConfig::new(1, 8, PllDivider::Div4))
            .pclk1(24.mhz())
            .pclk2(24.mhz())
            .freeze(&mut flash.acr, &mut pwr);
    }
}

#[exception]
unsafe fn DefaultHandler(irqn: i16) {
    // custom default handler
    // irqn is negative for Cortex-M exceptions
    // irqn is positive for device specific (line IRQ)
    panic!("Unregistred irq: {}", irqn);
}

#[exception]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    asm::bkpt();
    loop {}
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    //set_led(true);
    asm::bkpt();
    loop {}
}

#[no_mangle]
fn vApplicationStackOverflowHook(_pxTask: FreeRtosTaskHandle, _pcTaskName: FreeRtosCharPtr) {
    asm::bkpt();
}
