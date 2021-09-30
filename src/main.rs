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

mod threads;

//---------------------------------------------------------------

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

//---------------------------------------------------------------

#[entry]
fn main() -> ! {
    configure_clocks();
    /*
        Task::new()
            .name("thread")
            .stack_size(2548)
            .priority(TaskPriority(3))
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
            })
            .unwrap();
    */
    let _ = Task::new()
        .name("usbd")
        .stack_size(2048)
        .priority(TaskPriority(2))
        .start(move || threads::usbd::usbd(unsafe { stm32::Peripherals::steal() }))
        .unwrap();

    FreeRtosUtils::start_scheduler();
}

fn configure_clocks() {
    use stm32l4xx_hal::rcc::{PllConfig, PllDivider};

    let dp = unsafe { stm32::Peripherals::steal() };

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    {
        // set USB 48Mhz clock src
        // can be configured only before PLL enable
        let rcc = unsafe { &*stm32::RCC::ptr() };

        rcc.pllcfgr.modify(|_, w| unsafe {
            w.pllq()
                .bits(0b00) // /2
                .pllqen()
                .set_bit() // enable PLLQ
        });

        // PLLQ -> CLK48MHz
        unsafe { rcc.ccipr.modify(|_, w| w.clk48sel().bits(0b10)) };
    }

    let _ = rcc
        .cfgr
        // enable HSE
        .hse(
            12.mhz(),
            stm32l4xx_hal::rcc::CrystalBypass::Disable,
            stm32l4xx_hal::rcc::ClockSecuritySystem::Enable,
        )
        // set new cpu speed and PLL config
        .sysclk_with_pll(24.mhz(), PllConfig::new(1, 8, PllDivider::Div4))
        // set PLL source
        .pll_source(stm32l4xx_hal::rcc::PllSource::HSE)
        // set pclk1 speed
        .pclk1(24.mhz())
        // set pclk2 speed
        .pclk2(24.mhz())
        // apply changes
        .freeze(&mut flash.acr, &mut pwr);
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
