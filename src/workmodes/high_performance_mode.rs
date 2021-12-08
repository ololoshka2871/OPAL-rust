use alloc::sync::Arc;
use freertos_rust::{Duration, Mutex, Task, TaskPriority};
use stm32l4xx_hal::gpio::{
    Alternate, Floating, Input, Output, PushPull, AF1, AF10, PA0, PA11, PA12, PA8, PD10, PD13,
};
use stm32l4xx_hal::rcc::{PllConfig, PllDivider};
use stm32l4xx_hal::time::*;
use stm32l4xx_hal::{prelude::*, stm32};

use crate::sensors::freqmeter::master_counter;
use crate::support::{interrupt_controller::IInterruptController, InterruptController};
use crate::threads;
use crate::workmodes::common::ClockConfigProvider;

use super::WorkMode;

const PLL_CFG: (u32, u32, u32) = (3, 40, 2);
const APB1_DEVIDER: u32 = 8;
const APB2_DEVIDER: u32 = 8;

struct HighPerformanceClockConfigProvider;

impl ClockConfigProvider for HighPerformanceClockConfigProvider {
    fn core_frequency() -> Hertz {
        let f = crate::config::XTAL_FREQ * PLL_CFG.1 / (PLL_CFG.0 * PLL_CFG.2);
        Hertz(f)
    }

    fn apb1_frequency() -> Hertz {
        Hertz(Self::core_frequency().0 / APB1_DEVIDER)
    }

    fn apb2_frequency() -> Hertz {
        Hertz(Self::core_frequency().0 / APB2_DEVIDER)
    }

    // stm32_cube: if APB devider > 1, timers freq APB*2
    fn master_counter_frequency() -> Hertz {
        if APB1_DEVIDER > 1 {
            Hertz(Self::apb1_frequency().0 * 2)
        } else {
            Self::apb1_frequency()
        }
    }

    fn pll_config() -> PllConfig {
        let div = match PLL_CFG.2 {
            2 => PllDivider::Div2,
            4 => PllDivider::Div4,
            6 => PllDivider::Div6,
            8 => PllDivider::Div8,
            _ => panic!(),
        };
        PllConfig::new(PLL_CFG.0 as u8, PLL_CFG.1 as u8, div)
    }

    fn xtal2master_freq_multiplier() -> f32 {
        if APB1_DEVIDER > 1 {
            PLL_CFG.1 as f32 / (PLL_CFG.0 * PLL_CFG.2) as f32 / APB1_DEVIDER as f32 * 2.0
        } else {
            PLL_CFG.1 as f32 / (PLL_CFG.0 * PLL_CFG.2) as f32
        }
    }
}

pub struct HighPerformanceMode {
    rcc: stm32l4xx_hal::rcc::Rcc,
    flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>,
    pwr: stm32l4xx_hal::pwr::Pwr,

    clocks: Option<stm32l4xx_hal::rcc::Clocks>,

    usb: stm32l4xx_hal::stm32::USB,

    usb_dm: PA11<Alternate<AF10, Input<Floating>>>,
    usb_dp: PA12<Alternate<AF10, Input<Floating>>>,

    interrupt_controller: Arc<dyn IInterruptController>,

    crc: Arc<Mutex<stm32l4xx_hal::crc::Crc>>,

    in_p: PA8<Alternate<AF1, Input<Floating>>>,
    in_t: PA0<Alternate<AF1, Input<Floating>>>,
    en_p: PD13<Output<PushPull>>,
    en_t: PD10<Output<PushPull>>,
    dma1_ch2: stm32l4xx_hal::dma::dma1::C2,
    dma1_ch6: stm32l4xx_hal::dma::dma1::C6,
    timer1: stm32l4xx_hal::stm32l4::stm32l4x2::TIM1,
    timer2: stm32l4xx_hal::stm32l4::stm32l4x2::TIM2,

    sensor_command_queue: Arc<freertos_rust::Queue<threads::sensor_processor::Command>>,
}

impl WorkMode<HighPerformanceMode> for HighPerformanceMode {
    fn new(p: cortex_m::Peripherals, dp: stm32l4xx_hal::stm32l4::stm32l4x2::Peripherals) -> Self {
        use crate::config::GENERATOR_DISABLE_LVL;

        let mut rcc = dp.RCC.constrain();
        let ic = Arc::new(InterruptController::new(p.NVIC));
        let dma_channels = dp.DMA1.split(&mut rcc.ahb1);

        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
        let mut gpiod = dp.GPIOD.split(&mut rcc.ahb2);

        HighPerformanceMode {
            flash: Arc::new(Mutex::new(dp.FLASH.constrain()).unwrap()),
            crc: Arc::new(
                Mutex::new(super::configure_crc_module(dp.CRC.constrain(&mut rcc.ahb1))).unwrap(),
            ),

            usb: dp.USB,

            usb_dm: gpioa.pa11.into_af10(&mut gpioa.moder, &mut gpioa.afrh),
            usb_dp: gpioa.pa12.into_af10(&mut gpioa.moder, &mut gpioa.afrh),

            pwr: dp.PWR.constrain(&mut rcc.apb1r1),
            clocks: None,

            interrupt_controller: ic,

            rcc,

            in_p: gpioa.pa8.into_af1(&mut gpioa.moder, &mut gpioa.afrh),
            in_t: gpioa.pa0.into_af1(&mut gpioa.moder, &mut gpioa.afrl),

            en_p: gpiod.pd13.into_push_pull_output_with_state(
                &mut gpiod.moder,
                &mut gpiod.otyper,
                GENERATOR_DISABLE_LVL,
            ),
            en_t: gpiod.pd10.into_push_pull_output_with_state(
                &mut gpiod.moder,
                &mut gpiod.otyper,
                GENERATOR_DISABLE_LVL,
            ),

            dma1_ch2: dma_channels.2,
            dma1_ch6: dma_channels.6,
            timer1: dp.TIM1,
            timer2: dp.TIM2,

            sensor_command_queue: Arc::new(freertos_rust::Queue::new(5).unwrap()),
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
                    Hertz(crate::config::XTAL_FREQ), // onboard crystall
                    stm32l4xx_hal::rcc::CrystalBypass::Disable,
                    stm32l4xx_hal::rcc::ClockSecuritySystem::Enable,
                )
                .sysclk_with_pll(
                    HighPerformanceClockConfigProvider::core_frequency(),
                    HighPerformanceClockConfigProvider::pll_config(),
                )
                .pll_source(stm32l4xx_hal::rcc::PllSource::HSE)
                // if apb prescaler > 1 tomer clock = apb * 2
                .pclk1(HighPerformanceClockConfigProvider::apb1_frequency())
                .pclk2(HighPerformanceClockConfigProvider::apb2_frequency());

            core::mem::swap(&mut cfgr, work_cfgr);
        }

        setup_cfgr(&mut self.rcc.cfgr);

        let clocks = if let Ok(mut flash) = self.flash.lock(Duration::infinite()) {
            self.rcc.cfgr.freeze(&mut flash.acr, &mut self.pwr)
        } else {
            panic!()
        };

        configure_usb48();

        // stm32l433cc.pdf: fugure. 4
        master_counter::MasterCounter::init(
            HighPerformanceClockConfigProvider::master_counter_frequency(),
            self.interrupt_controller.clone(),
        );

        self.clocks = Some(clocks);
    }

    fn start_threads(self) -> Result<(), freertos_rust::FreeRtosError> {
        defmt::trace!("Creating usb thread...");
        let usbperith = threads::usbd::UsbdPeriph {
            usb: self.usb,
            pin_dp: self.usb_dp,
            pin_dm: self.usb_dm,
        };
        let ic = self.interrupt_controller.clone();
        Task::new()
            .name("Usbd")
            .stack_size(1024)
            .priority(TaskPriority(crate::config::USBD_TASK_PRIO))
            .start(move |_| {
                threads::usbd::usbd(usbperith, ic, crate::config::USB_INTERRUPT_PRIO)
            })?;

        defmt::trace!("Creating Sensors Processor thread...");
        let sp = threads::sensor_processor::SensorPerith {
            timer1: self.timer1,
            timer1_dma_ch: self.dma1_ch6,
            timer1_pin: self.in_p,
            en_1: self.en_p,

            timer2: self.timer2,
            timer2_dma_ch: self.dma1_ch2,
            timer2_pin: self.in_t,
            en_2: self.en_t,
        };
        let cq = self.sensor_command_queue.clone();
        let ic = self.interrupt_controller.clone();
        Task::new()
            .name("SensProc")
            .stack_size(1024)
            .priority(TaskPriority(crate::config::SENS_PROC_TASK_PRIO))
            .start(move |_| {
                threads::sensor_processor::sensor_processor(
                    sp,
                    cq,
                    ic,
                    HighPerformanceClockConfigProvider::xtal2master_freq_multiplier(),
                )
            })?;

        // ---
        crate::workmodes::common::create_monitor(self.clocks.unwrap().sysclk())?;

        #[cfg(debug_assertions)]
        {
            defmt::trace!("Creating pseudo-idle thread...");
            Task::new()
                .name("T_IDLE")
                .stack_size(48)
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
