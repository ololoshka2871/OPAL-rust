use freertos_rust::{Task, TaskPriority};
use stm32l4xx_hal::rcc::{PllConfig, PllDivider};
use stm32l4xx_hal::{prelude::*, stm32, stm32l4::stm32l4x2::Peripherals};

use crate::threads;

use super::WorkMode;

pub struct HighPerformanceMode {
    rcc: stm32l4xx_hal::rcc::Rcc,
    flash: stm32l4xx_hal::flash::Parts,
    pwr: Option<stm32l4xx_hal::pwr::Pwr>,

    clocks: Option<stm32l4xx_hal::rcc::Clocks>,

    usb: stm32l4xx_hal::stm32::USB,
    gpioa: Option<stm32l4xx_hal::gpio::gpioa::Parts>,
}

impl HighPerformanceMode {
    pub fn new(dp: Peripherals) -> Self {
        let mut res = HighPerformanceMode {
            rcc: dp.RCC.constrain(),
            flash: dp.FLASH.constrain(),
            usb: dp.USB,

            gpioa: None,
            pwr: None,
            clocks: None,
        };

        res.pwr = Some(dp.PWR.constrain(&mut res.rcc.apb1r1));
        res.gpioa = Some(dp.GPIOA.split(&mut res.rcc.ahb2));

        res
    }
}

impl WorkMode for HighPerformanceMode {
    //! Работа от внешнего кварца HSE = 12 MHz
    //! Установить частоту CPU = 80 MHz (12 / 3 * 40 / 2 == 80)
    //! USB работает от PLLSAI1Q = 48 MHz (12 / 3 * 24 / 2 == 48)
    fn configure_clock(&mut self) {
        fn configure_usb48() {
            let _rcc = unsafe { &*stm32::RCC::ptr() };

            // set USB 48Mhz clock src to PLLQ
            // can be configured only before PLL enable
            /*
            _rcc.pllcfgr.modify(|_, w| unsafe {
                w.pllq()
                    .bits(0b00) // PLLQ = PLL/2
                    .pllqen()
                    .set_bit() // enable PLLQ
            });
            */

            // PLLSAI1Q -> CLK48MHz
            unsafe { _rcc.ccipr.modify(|_, w| w.clk48sel().bits(0b00)) };
        }

        fn setut_cfgr(work_cfgr: &mut stm32l4xx_hal::rcc::CFGR) {
            let mut cfgr = unsafe {
                core::mem::MaybeUninit::<stm32l4xx_hal::rcc::CFGR>::zeroed().assume_init()
            };

            core::mem::swap(&mut cfgr, work_cfgr);

            let mut cfgr = cfgr
                .hsi48(true)
                .hse(
                    12.mhz(), // onboard crystall
                    stm32l4xx_hal::rcc::CrystalBypass::Disable,
                    stm32l4xx_hal::rcc::ClockSecuritySystem::Enable,
                )
                .sysclk_with_pll(
                    80.mhz(),                                // CPU clock
                    PllConfig::new(3, 40, PllDivider::Div2), // PLL config
                )
                .pll_source(stm32l4xx_hal::rcc::PllSource::HSE)
                .pclk1(80.mhz())
                .pclk2(80.mhz());

            core::mem::swap(&mut cfgr, work_cfgr);
        }

        configure_usb48();
        setut_cfgr(&mut self.rcc.cfgr);

        let clocks = self
            .rcc
            .cfgr
            .freeze(&mut self.flash.acr, self.pwr.as_mut().unwrap());

        defmt::info!(
            "Clock config: CPU={}, pclk1={}, pclk2={}, USB - HSI48",
            clocks.sysclk().0,
            clocks.pclk1().0,
            clocks.pclk2().0
        );

        self.clocks = Some(clocks);
    }

    fn start_threads(self) -> Result<(), freertos_rust::FreeRtosError> {
        if let Some(gpioa) = self.gpioa {
            defmt::trace!("Creating usb thread...");
            let usbperith = threads::usbd::UsbdPeriph {
                usb: self.usb,
                gpioa
            };

            Task::new()
                .stack_size(2048)
                .priority(TaskPriority(2))
                .start(move || threads::usbd::usbd(usbperith))?;
            Ok(())
        } else {
            defmt::panic!("GpioA not initialised!");
        }
    }
}
