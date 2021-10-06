use freertos_rust::{Task, TaskPriority};
use stm32l4xx_hal::rcc::{PllConfig, PllDivider};
use stm32l4xx_hal::{prelude::*, stm32, stm32l4::stm32l4x2::Peripherals};

use crate::threads;

use super::WorkMode;

pub struct HighPerformanceMode {
    rcc: stm32l4xx_hal::rcc::Rcc,
    flash: stm32l4xx_hal::flash::Parts,
    pwr: stm32l4xx_hal::pwr::Pwr,

    clocks: Option<stm32l4xx_hal::rcc::Clocks>,

    usb: stm32l4xx_hal::stm32::USB,
    gpioa: stm32l4xx_hal::gpio::gpioa::Parts,
}

impl WorkMode<HighPerformanceMode> for HighPerformanceMode {
    fn new(dp: Peripherals) -> Self {
        let mut rcc = dp.RCC.constrain();

        HighPerformanceMode {
            flash: dp.FLASH.constrain(),
            usb: dp.USB,

            gpioa: dp.GPIOA.split(&mut rcc.ahb2),
            pwr: dp.PWR.constrain(&mut rcc.apb1r1),
            clocks: None,

            rcc: rcc,
        }
    }

    // Работа от внешнего кварца HSE = 12 MHz
    // Установить частоту CPU = 80 MHz (12 / 3 * 40 / 2 == 80)
    // USB работает от PLLSAI1Q = 48 MHz (12 / 3 * 24 / 2 == 48)
    fn configure_clock(&mut self) {
        fn configure_usb48() {
            let _rcc = unsafe { &*stm32::RCC::ptr() };

            // set USB 48Mhz clock src to PLLSAI1Q
            // mast be configured only before PLL enable

            _rcc.cr.modify(|_, w| w.pllsai1on().clear_bit());
            while _rcc.cr.read().pllsai1rdy().bit_is_set() {}

            _rcc.pllsai1cfgr.modify(|_, w| unsafe {
                w.pllsai1n()
                    .bits(24) // * 24
                    .pllsai1q()
                    .bits(0b00) // /2
                    .pllsai1qen()
                    .set_bit() // enable PLLSAI1Q
            });

            _rcc.cr.modify(|_, w| w.pllsai1on().set_bit());
            while _rcc.cr.read().pllsai1rdy().bit_is_set() {}

            // PLLSAI1Q -> CLK48MHz
            unsafe { _rcc.ccipr.modify(|_, w| w.clk48sel().bits(0b01)) };
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

        setut_cfgr(&mut self.rcc.cfgr);

        let clocks = self.rcc.cfgr.freeze(&mut self.flash.acr, &mut self.pwr);
        configure_usb48();

        self.clocks = Some(clocks);
    }

    fn start_threads(self) -> Result<(), freertos_rust::FreeRtosError> {
        defmt::trace!("Creating usb thread...");
        let usbperith = threads::usbd::UsbdPeriph {
            usb: self.usb,
            gpioa: self.gpioa,
        };

        Task::new()
            .stack_size(2048)
            .priority(TaskPriority(2))
            .start(move || threads::usbd::usbd(usbperith))?;
        Ok(())
    }

    fn print_clock_config(&self) {
        super::common::print_clock_config(&self.clocks, "HSI48");
    }
}
