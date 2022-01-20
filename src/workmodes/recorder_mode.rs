use alloc::sync::Arc;
use freertos_rust::{Duration, Mutex, Task, TaskPriority};
use stm32l4xx_hal::{
    adc::ADC,
    gpio::{Alternate, Analog, Output, PushPull, Speed, PA0, PA1, PA8, PD0, PD10, PD13},
    prelude::*,
    rcc::{Enable, PllConfig, Reset},
    stm32,
    stm32l4::stm32l4x3::Peripherals,
    time::Hertz,
};

/*
use heatshrink_rust::decoder::HeatshrinkDecoder;
use heatshrink_rust::encoder::HeatshrinkEncoder;
*/

use crate::{
    sensors::freqmeter::master_counter,
    support::{interrupt_controller::IInterruptController, InterruptController},
    threads::{self, free_rtos_delay::FreeRtosDelay},
    workmodes::processing::RecorderProcessor,
};

use super::{common::ClockConfigProvider, output_storage::OutputStorage, WorkMode};

const PLL_CFG: (u32, u32, u32) = (1, 8, 8);
const APB1_DEVIDER: u32 = 1;
const APB2_DEVIDER: u32 = 1;

struct RecorderClockConfigProvider;

impl ClockConfigProvider for RecorderClockConfigProvider {
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
        PllConfig::new(
            PLL_CFG.0 as u8,
            PLL_CFG.1 as u8,
            super::common::to_pll_devider(PLL_CFG.2),
        )
    }

    fn xtal2master_freq_multiplier() -> f64 {
        if APB1_DEVIDER > 1 {
            PLL_CFG.1 as f64 / (PLL_CFG.0 * PLL_CFG.2) as f64 / APB1_DEVIDER as f64 * 2.0
        } else {
            PLL_CFG.1 as f64 / (PLL_CFG.0 * PLL_CFG.2) as f64
        }
    }
}

pub struct RecorderMode {
    rcc: stm32l4xx_hal::rcc::Rcc,
    flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>,
    pwr: stm32l4xx_hal::pwr::Pwr,

    clocks: Option<stm32l4xx_hal::rcc::Clocks>,

    interrupt_controller: Arc<dyn IInterruptController>,

    crc: Arc<Mutex<stm32l4xx_hal::crc::Crc>>,

    in_p: PA8<Alternate<PushPull, 1>>,
    in_t: PA0<Alternate<PushPull, 1>>,
    en_p: PD13<Output<PushPull>>,
    en_t: PD10<Output<PushPull>>,
    dma1_ch2: stm32l4xx_hal::dma::dma1::C2,
    dma1_ch6: stm32l4xx_hal::dma::dma1::C6,
    timer1: stm32l4xx_hal::stm32l4::stm32l4x3::TIM1,
    timer2: stm32l4xx_hal::stm32l4::stm32l4x3::TIM2,

    adc: stm32l4xx_hal::stm32::ADC1,
    adc_common: stm32l4xx_hal::device::ADC_COMMON,
    vbat_pin: PA1<Analog>,

    led_pin: PD0<Output<PushPull>>,

    sensor_command_queue: Arc<freertos_rust::Queue<threads::sensor_processor::Command>>,
}

impl WorkMode<RecorderMode> for RecorderMode {
    fn new(p: cortex_m::Peripherals, dp: Peripherals) -> Self {
        use crate::config::GENERATOR_DISABLE_LVL;

        let mut rcc = dp.RCC.constrain();

        let ic = Arc::new(InterruptController::new(p.NVIC));
        let dma_channels = dp.DMA1.split(&mut rcc.ahb1);

        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
        let mut gpiod = dp.GPIOD.split(&mut rcc.ahb2);

        RecorderMode {
            flash: Arc::new(Mutex::new(dp.FLASH.constrain()).unwrap()),
            crc: Arc::new(
                Mutex::new(super::configure_crc_module(dp.CRC.constrain(&mut rcc.ahb1))).unwrap(),
            ),

            pwr: dp.PWR.constrain(&mut rcc.apb1r1),
            clocks: None,

            interrupt_controller: ic,

            rcc,

            in_p: gpioa
                .pa8
                .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh)
                .set_speed(Speed::Low),
            in_t: gpioa
                .pa0
                .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl)
                .set_speed(Speed::Low),

            en_p: gpiod
                .pd13
                .into_push_pull_output_in_state(
                    &mut gpiod.moder,
                    &mut gpiod.otyper,
                    GENERATOR_DISABLE_LVL,
                )
                .set_speed(Speed::Low),
            en_t: gpiod
                .pd10
                .into_push_pull_output_in_state(
                    &mut gpiod.moder,
                    &mut gpiod.otyper,
                    GENERATOR_DISABLE_LVL,
                )
                .set_speed(Speed::Low),

            dma1_ch2: dma_channels.2,
            dma1_ch6: dma_channels.6,
            timer1: dp.TIM1,
            timer2: dp.TIM2,

            adc: dp.ADC1,
            adc_common: dp.ADC_COMMON,
            vbat_pin: gpioa.pa1.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),

            led_pin: gpiod
                .pd0
                .into_push_pull_output_in_state(
                    &mut gpiod.moder,
                    &mut gpiod.otyper,
                    crate::config::LED_DISABLE,
                )
                .set_speed(Speed::Low),

            sensor_command_queue: Arc::new(freertos_rust::Queue::new(40).unwrap()),
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
        crate::main_data_storage::init(self.flash());
    }

    // Работа от внешнего кварца HSE = 12 MHz
    // Установить частоту CPU = 12 MHz
    // USB не тактируется
    fn configure_clock(&mut self) {
        fn setup_cfgr(work_cfgr: &mut stm32l4xx_hal::rcc::CFGR) {
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
                    RecorderClockConfigProvider::core_frequency(),
                    RecorderClockConfigProvider::pll_config(),
                )
                .pll_source(stm32l4xx_hal::rcc::PllSource::HSE)
                // FIXME: master counter - max speed, input counters - slow down
                .pclk1(RecorderClockConfigProvider::apb1_frequency())
                .pclk2(RecorderClockConfigProvider::apb1_frequency());

            core::mem::swap(&mut cfgr, work_cfgr);
        }

        setup_cfgr(&mut self.rcc.cfgr);

        let clocks = if let Ok(mut flash) = self.flash.lock(Duration::infinite()) {
            self.rcc.cfgr.freeze(&mut flash.acr, &mut self.pwr)
        } else {
            panic!()
        };

        // stm32l433cc.pdf: fugure. 4
        master_counter::MasterCounter::init(
            RecorderClockConfigProvider::master_counter_frequency(),
            self.interrupt_controller.clone(),
        );

        self.clocks = Some(clocks);
    }

    fn start_threads(mut self) -> Result<(), freertos_rust::FreeRtosError> {
        let output = Arc::new(Mutex::new(OutputStorage::default()).unwrap());

        {
            use stm32l4xx_hal::adc::{Resolution, SampleTime};

            defmt::trace!("Creating Sensors Processor thread...");
            let mut delay = FreeRtosDelay {};
            {
                // Enable peripheral
                stm32::ADC1::enable(&mut self.rcc.ahb2);

                // Reset peripheral
                stm32::ADC1::reset(&mut self.rcc.ahb2);

                self.adc_common
                    .ccr
                    .modify(|_, w| unsafe { w.presc().bits(0b0100) });
            }
            let mut adc = ADC::new(
                self.adc,
                self.adc_common,
                &mut self.rcc.ahb2,
                &mut self.rcc.ccipr,
                &mut delay,
            );

            adc.set_sample_time(SampleTime::Cycles640_5);
            adc.set_resolution(Resolution::Bits12);

            let tcpu_ch = adc.enable_temperature(&mut delay);
            let v_ref = adc.enable_vref(&mut delay);

            let sp = threads::sensor_processor::SensorPerith {
                timer1: self.timer1,
                timer1_dma_ch: self.dma1_ch6,
                timer1_pin: self.in_p,
                en_1: self.en_p,

                timer2: self.timer2,
                timer2_dma_ch: self.dma1_ch2,
                timer2_pin: self.in_t,
                en_2: self.en_t,

                vbat_pin: self.vbat_pin,
                tcpu_ch: tcpu_ch,
                v_ref: v_ref,

                adc: adc,
            };
            let cq = self.sensor_command_queue.clone();
            let ic = self.interrupt_controller.clone();
            let mut processor = RecorderProcessor::new(
                output.clone(),
                self.sensor_command_queue.clone(),
                RecorderClockConfigProvider::xtal2master_freq_multiplier(),
                unsafe { self.clocks.unwrap_unchecked().sysclk() },
            );

            processor.start(
                crate::main_data_storage::cpu_flash_diff_writer::CpuFlashDiffWriter::new(
                    self.crc.clone(),
                ),
                self.led_pin,
            )?;

            Task::new()
                .name("SensProc")
                .stack_size(1024)
                .priority(TaskPriority(crate::config::SENS_PROC_TASK_PRIO))
                .start(move |_| {
                    threads::sensor_processor::sensor_processor(sp, cq, ic, processor)
                })?;
        }
        // --------------------------------------------------------------------

        crate::workmodes::common::create_monitor(
            unsafe { self.clocks.unwrap_unchecked().sysclk() },
            output.clone(),
        )?;

        super::common::create_pseudo_idle_task()?;

        Ok(())
    }

    fn print_clock_config(&self) {
        super::common::print_clock_config(&self.clocks, "OFF");
    }
}
