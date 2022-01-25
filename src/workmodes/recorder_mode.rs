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

use crate::{
    sensors::freqmeter::master_counter,
    support::{interrupt_controller::IInterruptController, InterruptController},
    threads::{self, free_rtos_delay::FreeRtosDelay},
    workmodes::processing::RecorderProcessor,
};

use super::{common::ClockConfigProvider, output_storage::OutputStorage, WorkMode};

const APB1_DEVIDER: u32 = 1;
const APB2_DEVIDER: u32 = 1;

struct RecorderClockConfigProvider;

impl ClockConfigProvider for RecorderClockConfigProvider {
    fn core_frequency() -> Hertz {
        Hertz(crate::config::FREERTOS_CONFIG_FREQ)
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
        unreachable!()
    }

    fn xtal2master_freq_multiplier() -> f64 {
        if APB1_DEVIDER > 1 {
            2.0 / (crate::config::XTAL_FREQ as f64 / crate::config::FREERTOS_CONFIG_FREQ as f64)
        } else {
            1.0 / (crate::config::XTAL_FREQ as f64 / crate::config::FREERTOS_CONFIG_FREQ as f64)
        }
    }
}

#[derive(Debug, PartialEq)]
/// HSE Configuration
struct HseConfig {
    /// Clock speed of HSE
    speed: u32,
    /// If the clock driving circuitry is bypassed i.e. using an oscillator, not a crystal or
    /// resonator
    bypass: stm32l4xx_hal::rcc::CrystalBypass,
    /// Clock Security System enable/disable
    css: stm32l4xx_hal::rcc::ClockSecuritySystem,
}

struct MyCFGR {
    hse: HseConfig,
    hclk: Option<u32>,
    pclk1: Option<u32>,
    pclk2: Option<u32>,
    sysclk: u32,
}

impl MyCFGR {
    fn new() -> Self {
        Self {
            hse: HseConfig {
                speed: 0,
                bypass: stm32l4xx_hal::rcc::CrystalBypass::Disable,
                css: stm32l4xx_hal::rcc::ClockSecuritySystem::Enable,
            },
            hclk: None,
            pclk1: None,
            pclk2: None,
            sysclk: 0,
        }
    }

    /// Add an HSE to the system
    pub fn hse<F>(
        mut self,
        freq: F,
        bypass: stm32l4xx_hal::rcc::CrystalBypass,
        css: stm32l4xx_hal::rcc::ClockSecuritySystem,
    ) -> Self
    where
        F: Into<Hertz>,
    {
        self.hse = HseConfig {
            speed: freq.into().0,
            bypass,
            css,
        };

        self
    }

    /// Sets a frequency for the AHB bus
    pub fn hclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.hclk = Some(freq.into().0);
        self
    }

    /// Sets the system (core) frequency
    pub fn sysclk<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.sysclk = freq.into().0;
        self
    }

    /// Sets a frequency for the APB1 bus
    pub fn pclk1<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.pclk1 = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB2 bus
    pub fn pclk2<F>(mut self, freq: F) -> Self
    where
        F: Into<Hertz>,
    {
        self.pclk2 = Some(freq.into().0);
        self
    }

    fn freeze(
        &self,
        _acr: &mut stm32l4xx_hal::flash::ACR,
        _pwr: &mut stm32l4xx_hal::pwr::Pwr,
    ) -> stm32l4xx_hal::rcc::Clocks {
        // Поскольку поля stm32l4xx_hal::rcc::Clocks приватные, делает точно такую же
        // структуру, заполняем её и трансмутируем тип core::mem::transmute()
        #[derive(Clone, Copy, Debug)]
        #[allow(dead_code)]
        struct Clocks {
            hclk: Hertz,
            hsi48: bool,
            msi: Option<stm32l4xx_hal::rcc::MsiFreq>,
            lsi: bool,
            lse: bool,
            pclk1: Hertz,
            pclk2: Hertz,
            ppre1: u8,
            ppre2: u8,
            sysclk: Hertz,
            pll_source: Option<stm32l4xx_hal::rcc::PllSource>,
        }

        let rcc = unsafe { &*stm32::RCC::ptr() };
        //
        // 1. Setup clocks
        //

        // If HSE is available, set it up

        rcc.cr.write(|w| {
            w.hseon().set_bit();

            if self.hse.bypass == stm32l4xx_hal::rcc::CrystalBypass::Enable {
                w.hsebyp().set_bit();
            }

            w
        });

        while rcc.cr.read().hserdy().bit_is_clear() {}

        // Setup CSS
        if self.hse.css == stm32l4xx_hal::rcc::ClockSecuritySystem::Enable {
            // Enable CSS
            rcc.cr.modify(|_, w| w.csson().set_bit());
        }

        assert!(self.sysclk <= 80_000_000);

        let (hpre_bits, hpre_div) = self
            .hclk
            .map(|hclk| match self.sysclk / hclk {
                // From p 194 in RM0394
                0 => unreachable!(),
                1 => (0b0000, 1),
                2 => (0b1000, 2),
                3..=5 => (0b1001, 4),
                6..=11 => (0b1010, 8),
                12..=39 => (0b1011, 16),
                40..=95 => (0b1100, 64),
                96..=191 => (0b1101, 128),
                192..=383 => (0b1110, 256),
                _ => (0b1111, 512),
            })
            .unwrap_or((0b0000, 1));

        let hclk = self.sysclk / hpre_div;

        assert!(hclk <= self.sysclk);

        let (ppre1_bits, ppre1) = self
            .pclk1
            .map(|pclk1| match hclk / pclk1 {
                // From p 194 in RM0394
                0 => unreachable!(),
                1 => (0b000, 1),
                2 => (0b100, 2),
                3..=5 => (0b101, 4),
                6..=11 => (0b110, 8),
                _ => (0b111, 16),
            })
            .unwrap_or((0b000, 1));

        let pclk1 = hclk / ppre1 as u32;

        assert!(pclk1 <= self.sysclk);

        let (ppre2_bits, ppre2) = self
            .pclk2
            .map(|pclk2| match hclk / pclk2 {
                // From p 194 in RM0394
                0 => unreachable!(),
                1 => (0b000, 1),
                2 => (0b100, 2),
                3..=5 => (0b101, 4),
                6..=11 => (0b110, 8),
                _ => (0b111, 16),
            })
            .unwrap_or((0b000, 1));

        let pclk2 = hclk / ppre2 as u32;

        assert!(pclk2 <= self.sysclk);

        // adjust flash wait states
        unsafe {
            (*stm32::FLASH::ptr()).acr.write(|w| {
                w.latency().bits(if hclk <= 16_000_000 {
                    0b000
                } else if hclk <= 32_000_000 {
                    0b001
                } else if hclk <= 48_000_000 {
                    0b010
                } else if hclk <= 64_000_000 {
                    0b011
                } else {
                    0b100
                })
            })
        }

        let sysclk_src_bits = 0b10; // HSE

        // HSE: HSE selected as system clock
        rcc.cfgr.write(|w| unsafe {
            w.ppre2()
                .bits(ppre2_bits)
                .ppre1()
                .bits(ppre1_bits)
                .hpre()
                .bits(hpre_bits)
                .sw()
                .bits(sysclk_src_bits)
        });

        while rcc.cfgr.read().sws().bits() != sysclk_src_bits {}

        //
        // 3. Shutdown unused clocks that have auto-started
        //

        // MSI always starts on reset
        {
            rcc.cr
                .modify(|_, w| w.msion().clear_bit().msipllen().clear_bit())
        }

        //
        // 4. Clock setup done!
        //

        unsafe {
            core::mem::transmute(Clocks {
                hclk: Hertz(hclk),
                lsi: false,
                lse: false,
                msi: None,
                hsi48: false,
                pclk1: Hertz(pclk1),
                pclk2: Hertz(pclk2),
                ppre1,
                ppre2,
                sysclk: Hertz(self.sysclk),
                pll_source: None,
            })
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
    scb: cortex_m::peripheral::SCB,

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
            scb: p.SCB,

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
        fn setup_cfgr() -> MyCFGR {
            MyCFGR::new()
                // TODO: constants
                .hse(
                    Hertz(crate::config::XTAL_FREQ), // onboard crystall
                    stm32l4xx_hal::rcc::CrystalBypass::Disable,
                    stm32l4xx_hal::rcc::ClockSecuritySystem::Enable,
                )
                .sysclk(Hertz(crate::config::XTAL_FREQ))
                .hclk(RecorderClockConfigProvider::core_frequency())
                .pclk1(RecorderClockConfigProvider::apb1_frequency())
                .pclk2(RecorderClockConfigProvider::apb2_frequency())
        }

        let cfgr = setup_cfgr();

        let clocks = if let Ok(mut flash) = self.flash.lock(Duration::infinite()) {
            cfgr.freeze(&mut flash.acr, &mut self.pwr)
        } else {
            panic!()
        };

        // low power run (F <= 2MHz) (на 12 MHz выйгрыш около 200мкА)
        unsafe {
            (*stm32l4xx_hal::device::PWR::ptr())
                .cr1
                .modify(|_, w| w.lpr().set_bit())
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
                self.scb,
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
