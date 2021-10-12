use freertos_rust::{Duration, Task, TaskPriority};
use stm32l4xx_hal::{
    prelude::*,
    rcc::{PllConfig, PllDivider},
    stm32l4::stm32l4x2::Peripherals,
};

use heatshrink_rust::decoder::HeatshrinkDecoder;
use heatshrink_rust::encoder::HeatshrinkEncoder;

use crate::{threads, workmodes::common::{calc_monitoring_period, enable_dma_clocking}};

use super::WorkMode;

pub struct PowerSaveMode {
    rcc: stm32l4xx_hal::rcc::Rcc,
    flash: stm32l4xx_hal::flash::Parts,
    pwr: Option<stm32l4xx_hal::pwr::Pwr>,

    clocks: Option<stm32l4xx_hal::rcc::Clocks>,
}

impl WorkMode<PowerSaveMode> for PowerSaveMode {
    fn new(_p: cortex_m::Peripherals, dp: Peripherals) -> Self {
        let mut res = PowerSaveMode {
            rcc: dp.RCC.constrain(),
            flash: dp.FLASH.constrain(),
            pwr: None,
            clocks: None,
        };

        res.pwr = Some(dp.PWR.constrain(&mut res.rcc.apb1r1));

        res
    }

    // Работа от внешнего кварца HSE = 12 MHz
    // Установить частоту CPU = 12 MHz
    // USB не тактируется
    fn configure_clock(&mut self) {
        fn setut_cfgr(work_cfgr: &mut stm32l4xx_hal::rcc::CFGR) {
            let mut cfgr = unsafe {
                core::mem::MaybeUninit::<stm32l4xx_hal::rcc::CFGR>::zeroed().assume_init()
            };

            core::mem::swap(&mut cfgr, work_cfgr);

            let mut cfgr = cfgr
                .hse(
                    12.mhz(), // onboard crystall
                    stm32l4xx_hal::rcc::CrystalBypass::Disable,
                    stm32l4xx_hal::rcc::ClockSecuritySystem::Enable,
                )
                // FIXME: Don't use PLL, dirrectly connect HSE to CPU (see freeze())
                .sysclk_with_pll(
                    12.mhz(),                               // CPU clock
                    PllConfig::new(1, 8, PllDivider::Div8), // PLL config
                )
                .pll_source(stm32l4xx_hal::rcc::PllSource::HSE)

                // FIXME: master counter - max speed, input counters - slow down
                .pclk1(12.mhz())
                .pclk2(12.mhz());

            core::mem::swap(&mut cfgr, work_cfgr);
        }

        setut_cfgr(&mut self.rcc.cfgr);

        let clocks = self
            .rcc
            .cfgr
            .freeze(&mut self.flash.acr, self.pwr.as_mut().unwrap());

        enable_dma_clocking();
            
        self.clocks = Some(clocks);
    }

    fn start_threads(self) -> Result<(), freertos_rust::FreeRtosError> {
        {
        Task::new()
            .name("hs-test")
            .stack_size(2548)
            .priority(TaskPriority(3))
            .start(move |_| {
                let src = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];

                let mut it_src = src.iter().map(|a| *a);

                let mut enc = HeatshrinkEncoder::from_source(&mut it_src);
                let dec = HeatshrinkDecoder::from_source(&mut enc);

                for (i, b) in dec.enumerate() {
                    defmt::debug!("decoded[{}] = {:X}", i, b);
                }
            })?;
        }
        // ---
        {
            defmt::trace!("Creating monitor thread...");
            let monitoring_period =
                calc_monitoring_period(Duration::ms(1000), self.clocks.unwrap().sysclk());
            Task::new()
                .name("Monitord")
                .stack_size(1024)
                .priority(TaskPriority(1))
                .start(move |_| threads::monitor::monitord(monitoring_period))?;
        }
        Ok(())
    }

    fn print_clock_config(&self) {
        super::common::print_clock_config(&self.clocks, "OFF");
    }
}
