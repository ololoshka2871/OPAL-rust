use alloc::sync::Arc;
use freertos_rust::{Duration, Mutex, Task, TaskPriority};
use stm32l4xx_hal::rcc::{PllConfig, PllDivider};
use stm32l4xx_hal::{prelude::*, stm32};

use crate::sensors::freqmeter::master_counter;
use crate::support::{interrupt_controller::IInterruptController, InterruptController};
use crate::threads;

use super::WorkMode;

pub struct HighPerformanceMode {
    rcc: stm32l4xx_hal::rcc::Rcc,
    flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>,
    pwr: stm32l4xx_hal::pwr::Pwr,

    clocks: Option<stm32l4xx_hal::rcc::Clocks>,

    usb: stm32l4xx_hal::stm32::USB,
    gpioa: stm32l4xx_hal::gpio::gpioa::Parts,

    interrupt_controller: Arc<dyn IInterruptController>,

    crc: Arc<Mutex<stm32l4xx_hal::crc::Crc>>,
}

impl WorkMode<HighPerformanceMode> for HighPerformanceMode {
    fn new(p: cortex_m::Peripherals, dp: stm32l4xx_hal::stm32l4::stm32l4x2::Peripherals) -> Self {
        let mut rcc = dp.RCC.constrain();
        let ic = Arc::new(InterruptController::new(p.NVIC));

        HighPerformanceMode {
            flash: Arc::new(Mutex::new(dp.FLASH.constrain()).unwrap()),
            crc: Arc::new(
                Mutex::new(super::configure_crc_module(dp.CRC.constrain(&mut rcc.ahb1))).unwrap(),
            ),

            usb: dp.USB,

            gpioa: dp.GPIOA.split(&mut rcc.ahb2),
            pwr: dp.PWR.constrain(&mut rcc.apb1r1),
            clocks: None,

            interrupt_controller: ic,

            rcc,
        }
    }

    fn flash(&mut self) -> Arc<Mutex<stm32l4xx_hal::flash::Parts>> {
        self.flash.clone()
    }

    fn crc(&mut self) -> Arc<Mutex<stm32l4xx_hal::crc::Crc>> {
        self.crc.clone()
    }

    fn ini_static(&mut self) {
        crate::settings::init(self.flash(), self.crc());
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

        fn setup_cfgr(work_cfgr: &mut stm32l4xx_hal::rcc::CFGR) {
            let mut cfgr = unsafe {
                core::mem::MaybeUninit::<stm32l4xx_hal::rcc::CFGR>::zeroed().assume_init()
            };

            core::mem::swap(&mut cfgr, work_cfgr);

            let mut cfgr = cfgr
                .hsi48(true)
                .hse(
                    crate::workmodes::common::HSE_FREQ, // onboard crystall
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

        setup_cfgr(&mut self.rcc.cfgr);

        let clocks = if let Ok(mut flash) = self.flash.lock(Duration::infinite()) {
            self.rcc.cfgr.freeze(&mut flash.acr, &mut self.pwr)
        } else {
            panic!()
        };

        configure_usb48();

        master_counter::MasterCounter::init(self.interrupt_controller.clone());

        self.clocks = Some(clocks);
    }

    fn start_threads(self) -> Result<(), freertos_rust::FreeRtosError> {
        defmt::trace!("Creating usb thread...");
        let usbperith = threads::usbd::UsbdPeriph {
            usb: self.usb,
            gpioa: self.gpioa,
        };
        let ic = self.interrupt_controller.clone();
        Task::new()
            .name("Usbd")
            .stack_size(1024)
            .priority(TaskPriority(crate::config::USBD_TASK_PRIO))
            .start(move |_| {
                threads::usbd::usbd(usbperith, ic, crate::config::USB_INTERRUPT_PRIO)
            })?;

        // ---
        crate::workmodes::common::create_monitor(self.clocks.unwrap().sysclk())?;

        #[cfg(debug_assertions)]
        {
            defmt::trace!("Creating pseudo-idle thread...");
            Task::new()
                .name("T_IDLE")
                .stack_size(128)
                .priority(TaskPriority(crate::config::PSEOUDO_IDLE_TASK_PRIO))
                .start(move |_| loop {
                    unsafe {
                        freertos_rust::freertos_rs_isr_yield();
                    }
                })?;
        }

        Ok(())
    }

    fn print_clock_config(&self) {
        super::common::print_clock_config(&self.clocks, "HSI48");
    }
}
