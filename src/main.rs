#![no_std]
#![no_main]
// For allocator
#![feature(lang_items)]
#![feature(alloc_error_handler)]

mod support;

use cortex_m_rt::entry;
use freertos_rust::*;

//use cortex_m_log::printer::{semihosting, Printer};

//use heatshrink_rust::decoder::HeatshrinkDecoder;
//use heatshrink_rust::encoder::HeatshrinkEncoder;

use stm32l4xx_hal::{prelude::*, stm32};

mod threads;

//---------------------------------------------------------------

#[global_allocator]
static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;

//---------------------------------------------------------------

#[entry]
fn main() -> ! {
    defmt::trace!("Start up");
    configure_clocks();
    defmt::trace!("Clocks configured");
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
    defmt::trace!("Creating usb thread...");
    let r = Task::new()
        .name("usbd")
        .stack_size(2048)
        .priority(TaskPriority(2))
        .start(move || threads::usbd::usbd(unsafe { stm32::Peripherals::steal() }));
    defmt::trace!("Result: {}", r.is_ok());

    defmt::trace!("Starting FreeRTOS sharuler");
    FreeRtosUtils::start_scheduler();
}

fn configure_clocks() {
    use stm32l4xx_hal::rcc::{PllConfig, PllDivider};

    let dp = unsafe { stm32::Peripherals::steal() };

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    defmt::info!("Set CLK48MHz source to PLLQ/2");
    {
        // set USB 48Mhz clock src to PLLQ
        // can be configured only before PLL enable
        let _rcc = unsafe { &*stm32::RCC::ptr() };

        _rcc.pllcfgr.modify(|_, w| unsafe {
            w.pllq()
                .bits(0b00) // PLLQ = PLL/2
                .pllqen()
                .set_bit() // enable PLLQ
        });

        // PLLQ -> CLK48MHz
        unsafe { _rcc.ccipr.modify(|_, w| w.clk48sel().bits(0b10)) };
    }

    let clocks = rcc
        .cfgr
        .hse(
            12.mhz(), // onboard crystall
            stm32l4xx_hal::rcc::CrystalBypass::Disable,
            stm32l4xx_hal::rcc::ClockSecuritySystem::Enable,
        )
        .sysclk_with_pll(
            24.mhz(),                               // CPU clock
            PllConfig::new(1, 8, PllDivider::Div4), // PLL config
        )
        .pll_source(stm32l4xx_hal::rcc::PllSource::HSE)
        .pclk1(24.mhz())
        .pclk2(24.mhz())
        .freeze(&mut flash.acr, &mut pwr);

    defmt::info!(
        "Clock config: CPU={}, pclk1={}, pclk2={}",
        clocks.sysclk().0,
        clocks.pclk1().0,
        clocks.pclk2().0
    );
}
