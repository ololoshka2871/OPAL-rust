use alloc::sync::Arc;
use freertos_rust::{Duration, Mutex, Queue, Task, TaskPriority};

#[allow(unused_imports)]
use stm32l4xx_hal::gpio::{
    Alternate, Analog, Output, PushPull, Speed, PA0, PA1, PA11, PA12, PA2, PA3, PA6, PA7, PA8, PB0,
    PC10, PD10, PD11, PD13, PE12,
};
use stm32l4xx_hal::{
    adc::ADC,
    prelude::*,
    rcc::{Enable, PllConfig, Reset},
    stm32,
    time::Hertz,
};

use crate::sensors::freqmeter::master_counter;
use crate::support::{interrupt_controller::IInterruptController, InterruptController};
use crate::threads;
use crate::threads::free_rtos_delay::FreeRtosDelay;
use crate::threads::sensor_processor::Command;
use crate::workmodes::{common::ClockConfigProvider, processing::HighPerformanceProcessor};

use super::{output_storage::OutputStorage, WorkMode};

// /PD *M /AD
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

#[allow(unused)]
pub struct HighPerformanceMode {
    rcc: stm32l4xx_hal::rcc::Rcc,
    flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>,
    pwr: stm32l4xx_hal::pwr::Pwr,

    clocks: Option<stm32l4xx_hal::rcc::Clocks>,

    usb: stm32l4xx_hal::stm32::USB,

    usb_dm: PA11<Alternate<PushPull, 10>>,
    usb_dp: PA12<Alternate<PushPull, 10>>,

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

    led_pin: PC10<Output<PushPull>>,

    adc: stm32l4xx_hal::stm32::ADC1,
    adc_common: stm32l4xx_hal::device::ADC_COMMON,
    vbat_pin: PA1<Analog>,

    #[cfg(not(feature = "no-flash"))]
    qspi: qspi_stm32lx3::qspi::Qspi<(
        PA3<Alternate<PushPull, 10>>,
        PA2<Alternate<PushPull, 10>>,
        PE12<Alternate<PushPull, 10>>,
        PB0<Alternate<PushPull, 10>>,
        PA7<Alternate<PushPull, 10>>,
        PA6<Alternate<PushPull, 10>>,
    )>,
    #[cfg(not(feature = "no-flash"))]
    flash_reset_pin: PD11<Output<PushPull>>,

    sensor_command_queue: Arc<freertos_rust::Queue<threads::sensor_processor::Command>>,

    output: Arc<Mutex<OutputStorage>>,
}

impl WorkMode<HighPerformanceMode> for HighPerformanceMode {
    fn new(p: cortex_m::Peripherals, dp: stm32l4xx_hal::stm32l4::stm32l4x3::Peripherals) -> Self {
        use crate::config::GENERATOR_DISABLE_LVL;

        let mut rcc = dp.RCC.constrain();
        let ic = Arc::new(InterruptController::new(p.NVIC));
        let dma_channels = dp.DMA1.split(&mut rcc.ahb1);

        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
        let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
        let mut gpiod = dp.GPIOD.split(&mut rcc.ahb2);

        #[cfg(not(feature = "no-flash"))]
        let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
        #[cfg(not(feature = "no-flash"))]
        let mut gpioe = dp.GPIOE.split(&mut rcc.ahb2);

        #[cfg(not(feature = "no-flash"))]
        let (qspi, flash_reset_pin) = super::common::create_qspi(
            (
                gpioa
                    .pa3
                    .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl),
                gpioa
                    .pa2
                    .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl),
                gpioe
                    .pe12
                    .into_alternate(&mut gpioe.moder, &mut gpioe.otyper, &mut gpioe.afrh),
                gpiob
                    .pb0
                    .into_alternate(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl),
                gpioa
                    .pa7
                    .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl),
                gpioa
                    .pa6
                    .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl),
            ),
            gpiod.pd11.into_push_pull_output_in_state(
                &mut gpiod.moder,
                &mut gpiod.otyper,
                PinState::Low,
            ),
            &mut rcc.ahb3,
        );

        HighPerformanceMode {
            flash: Arc::new(Mutex::new(dp.FLASH.constrain()).unwrap()),
            crc: Arc::new(
                Mutex::new(super::configure_crc_module(dp.CRC.constrain(&mut rcc.ahb1))).unwrap(),
            ),

            usb: dp.USB,

            usb_dm: gpioa
                .pa11
                .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh)
                .set_speed(Speed::VeryHigh),
            usb_dp: gpioa
                .pa12
                .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh)
                .set_speed(Speed::VeryHigh),

            pwr: dp.PWR.constrain(&mut rcc.apb1r1),
            clocks: None,

            interrupt_controller: ic,

            #[cfg(not(feature = "no-flash"))]
            qspi,
            #[cfg(not(feature = "no-flash"))]
            flash_reset_pin,

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

            led_pin: gpioc
                .pc10
                .into_push_pull_output_in_state(&mut gpioc.moder, &mut gpioc.otyper, PinState::High)
                .set_speed(Speed::Low),

            adc: dp.ADC1,
            adc_common: dp.ADC_COMMON,
            vbat_pin: gpioa.pa1.into_analog(&mut gpioa.moder, &mut gpioa.pupdr),

            sensor_command_queue: Arc::new(freertos_rust::Queue::new(15).unwrap()),

            output: Arc::new(Mutex::new(OutputStorage::default()).unwrap()),
        }
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
                .hsi48(false)
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

        // stm32l433cc.pdf: figure. 4
        master_counter::MasterCounter::init(
            HighPerformanceClockConfigProvider::master_counter_frequency(),
            self.interrupt_controller.clone(),
        );

        self.clocks = Some(clocks);
    }

    fn start_threads(mut self) -> Result<(), freertos_rust::FreeRtosError> {
        let sys_clk = unsafe { self.clocks.unwrap_unchecked().hclk() };

        crate::support::led::led_init(self.led_pin);

        #[cfg(not(feature = "no-flash"))]
        crate::main_data_storage::init(self.qspi, sys_clk, self.flash_reset_pin);

        {
            defmt::trace!("Creating usb thread...");
            let usbperith = threads::usbd::UsbdPeriph {
                usb: self.usb,
                pin_dp: self.usb_dp,
                pin_dm: self.usb_dm,
            };
            let cq = self.sensor_command_queue.clone();
            let ic = self.interrupt_controller.clone();
            let output = self.output.clone();
            Task::new()
                .name("Usbd")
                .stack_size(1024)
                .priority(TaskPriority(crate::config::USBD_TASK_PRIO))
                .start(move |_| {
                    threads::usbd::usbd(
                        usbperith,
                        ic,
                        crate::config::USB_INTERRUPT_PRIO,
                        output,
                        cq,
                    )
                })?;
        }
        // --------------------------------------------------------------------
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
            let processor = HighPerformanceProcessor::new(
                self.output.clone(),
                HighPerformanceClockConfigProvider::xtal2master_freq_multiplier(),
                sys_clk,
            );
            Task::new()
                .name("SensProc")
                .stack_size(1024)
                .priority(TaskPriority(crate::config::SENS_PROC_TASK_PRIO))
                .start(move |_| {
                    threads::sensor_processor::sensor_processor(sp, cq, ic, processor, sys_clk)
                })?;
        }
        // --------------------------------------------------------------------

        crate::workmodes::common::create_monitor(sys_clk, self.output.clone())?;

        //super::common::create_pseudo_idle_task()?;

        enable_selected_channels(self.sensor_command_queue.as_ref());

        Ok(())
    }

    fn print_clock_config(&self) {
        super::common::print_clock_config(&self.clocks, "HSI48");
    }

    fn flash(&mut self) -> Arc<Mutex<stm32l4xx_hal::flash::Parts>> {
        self.flash.clone()
    }

    fn crc(&mut self) -> Arc<Mutex<stm32l4xx_hal::crc::Crc>> {
        self.crc.clone()
    }
}

pub fn enable_selected_channels(cq: &Queue<Command>) {
    use crate::threads::sensor_processor::{AChannel, Channel, FChannel};
    use freertos_rust::FreeRtosError;

    let _ = crate::settings::settings_action::<_, _, _, FreeRtosError>(
        Duration::infinite(),
        |(ws, _)| {
            let flags = [
                (Channel::FChannel(FChannel::Pressure), ws.P_enabled),
                (Channel::FChannel(FChannel::Temperature), ws.T_enabled),
                (Channel::AChannel(AChannel::TCPU), ws.TCPUEnabled),
                (Channel::AChannel(AChannel::Vbat), ws.VBatEnabled),
            ];
            for (c, enabled) in flags.iter() {
                if *enabled {
                    cq.send(Command::Start(*c, 0), Duration::infinite())
                } else {
                    cq.send(Command::Stop(*c), Duration::infinite())
                }?;
            }

            Ok(())
        },
    )
    .map_err(|e| panic!("Failed to read channel enable: {:?}", e));
}
