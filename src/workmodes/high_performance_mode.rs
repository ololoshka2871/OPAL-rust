use alloc::sync::Arc;

use freertos_rust::{Mutex, Task, TaskPriority};

use stm32f1xx_hal::{
    device::{GPIOB, TIM1, TIM4},
    prelude::*,
    rcc::{HPre, PPre},
    time::Hertz,
    timer::{PwmChannel, Timer},
};

use stm32f1xx_hal::gpio::{
    Floating, Input, Output, PushPull, PA0, PA1, PA11, PA12, PA15, PA2, PA3, PA4, PA5, PA6, PA7,
    PA9, PB3, PB4, PB5, PB6, PC13, PC14, PC15,
};

use crate::control::{
    laser,
    xy2_100::{self, XY2_100Interface},
};
use crate::gcode::Request;
use crate::support::InterruptController;
use crate::threads;
use crate::workmodes::common::ClockConfigProvider;

use super::WorkMode;

mod clock_config_72;
use clock_config_72::{
    ADC_DEVIDER, AHB_DEVIDER, APB1_DEVIDER, APB2_DEVIDER, PLL_MUL, PLL_P_DIV, USB_DEVIDER,
};

struct HighPerformanceClockConfigProvider;

impl HighPerformanceClockConfigProvider {
    fn ahb_dev2val(ahb_dev: HPre) -> u32 {
        match ahb_dev {
            HPre::DIV1 => 1,
            HPre::DIV2 => 2,
            HPre::DIV4 => 4,
            HPre::DIV8 => 8,
            HPre::DIV16 => 16,
            HPre::DIV64 => 64,
            HPre::DIV128 => 128,
            HPre::DIV256 => 256,
            HPre::DIV512 => 512,
        }
    }

    fn apb_dev2val(apb_dev: PPre) -> u32 {
        match apb_dev {
            PPre::DIV1 => 1,
            PPre::DIV2 => 2,
            PPre::DIV4 => 4,
            PPre::DIV8 => 8,
            PPre::DIV16 => 16,
        }
    }

    fn pll_mul_bits(mul: u32) -> u8 {
        (mul - 2) as u8
    }

    fn ppl_div2val(div: stm32f1xx_hal::device::rcc::cfgr::PLLXTPRE_A) -> u32 {
        match div {
            stm32f1xx_hal::device::rcc::cfgr::PLLXTPRE_A::DIV1 => 1,
            stm32f1xx_hal::device::rcc::cfgr::PLLXTPRE_A::DIV2 => 2,
        }
    }

    fn freeze(_acr: &mut stm32f1xx_hal::flash::ACR) -> stm32f1xx_hal::rcc::Clocks {
        use stm32f1xx_hal::time::MHz;

        let cfg = Self::to_config();

        let clocks = cfg.get_clocks();
        // adjust flash wait states
        let acr = unsafe { &*stm32f1xx_hal::device::FLASH::ptr() };
        unsafe {
            acr.acr.write(|w| {
                w.latency().bits(if clocks.sysclk() <= MHz(24) {
                    0b000
                } else if clocks.sysclk() <= MHz(48) {
                    0b001
                } else {
                    0b010
                })
            })
        }

        let rcc = unsafe { &*stm32f1xx_hal::device::RCC::ptr() };

        if cfg.hse.is_some() {
            // enable HSE and wait for it to be ready

            rcc.cr.modify(|_, w| w.hseon().set_bit());

            while rcc.cr.read().hserdy().bit_is_clear() {}
        }

        if let Some(pllmul_bits) = cfg.pllmul {
            // enable PLL and wait for it to be ready

            #[allow(unused_unsafe)]
            rcc.cfgr
                .modify(|_, w| unsafe { w.pllxtpre().variant(PLL_P_DIV) });

            #[allow(unused_unsafe)]
            rcc.cfgr.modify(|_, w| unsafe {
                w.pllmul().bits(pllmul_bits).pllsrc().bit(cfg.hse.is_some())
            });

            rcc.cr.modify(|_, w| w.pllon().set_bit());

            while rcc.cr.read().pllrdy().bit_is_clear() {}
        }

        rcc.cfgr.modify(|_, w| unsafe {
            w.adcpre().variant(cfg.adcpre);
            w.ppre2()
                .bits(cfg.ppre2 as u8)
                .ppre1()
                .bits(cfg.ppre1 as u8)
                .hpre()
                .bits(cfg.hpre as u8)
                .usbpre()
                .variant(cfg.usbpre)
                .sw()
                .bits(if cfg.pllmul.is_some() {
                    // PLL
                    0b10
                } else if cfg.hse.is_some() {
                    // HSE
                    0b1
                } else {
                    // HSI
                    0b0
                })
        });

        clocks
    }
}

impl ClockConfigProvider for HighPerformanceClockConfigProvider {
    fn core_frequency() -> Hertz {
        let f = crate::config::XTAL_FREQ / Self::ppl_div2val(PLL_P_DIV) * PLL_MUL
            / Self::ahb_dev2val(AHB_DEVIDER);
        Hertz::Hz(f)
    }

    fn apb1_frequency() -> Hertz {
        Hertz::Hz(Self::core_frequency().to_Hz() / Self::apb_dev2val(APB1_DEVIDER))
    }

    fn apb2_frequency() -> Hertz {
        Hertz::Hz(Self::core_frequency().to_Hz() / Self::apb_dev2val(APB2_DEVIDER))
    }

    // stm32_cube: if APB devider > 1, timers freq APB*2
    fn master_counter_frequency() -> Hertz {
        Self::apb2_frequency() // TIM1 -> master
    }

    fn xtal2master_freq_multiplier() -> f32 {
        PLL_MUL as f32
            / (Self::ppl_div2val(PLL_P_DIV)
                * Self::ahb_dev2val(AHB_DEVIDER)
                * Self::apb_dev2val(APB2_DEVIDER)) as f32
    }

    fn to_config() -> stm32f1xx_hal::rcc::Config {
        stm32f1xx_hal::rcc::Config {
            hse: Some(crate::config::XTAL_FREQ),
            pllmul: Some(Self::pll_mul_bits(PLL_MUL)),
            hpre: AHB_DEVIDER,
            ppre1: APB1_DEVIDER,
            ppre2: APB2_DEVIDER,
            usbpre: USB_DEVIDER,
            adcpre: ADC_DEVIDER,
        }
    }
}

//-----------------------------------------------------------------------------

crate::simple_parallel_output_bus! { LaserDataBus: u8 =>
    (
        pin PA0<Output<PushPull>>,
        pin PA1<Output<PushPull>>,
        pin PA2<Output<PushPull>>,
        pin PA3<Output<PushPull>>,
        pin PA4<Output<PushPull>>,
        pin PA5<Output<PushPull>>,
        pin PA6<Output<PushPull>>,
        pin PA7<Output<PushPull>>
    )
}

crate::simple_parallel_input_bus! { LaserAlarmBus: u8 =>
    (
        pin PC13<Input<Floating>>,
        pin PC14<Input<Floating>>,
        pin PC15<Input<Floating>>
    )
}

//-----------------------------------------------------------------------------

#[allow(unused)]
pub struct HighPerformanceMode {
    flash: Arc<Mutex<stm32f1xx_hal::flash::Parts>>,

    clocks: stm32f1xx_hal::rcc::Clocks,

    usb: stm32f1xx_hal::stm32::USB,
    usb_dm: PA11<Input<Floating>>,
    usb_dp: PA12<Input<Floating>>,
    usb_pull_up: PA15<Output<PushPull>>,

    interrupt_controller: Arc<InterruptController>,

    crc: Arc<Mutex<stm32f1xx_hal::crc::Crc>>,

    galvo_ctrl: xy2_100::XY2_100<
        stm32f1xx_hal::device::TIM2,
        stm32f1xx_hal::dma::dma1::C2,
        (
            PB3<Output<PushPull>>,
            PB4<Output<PushPull>>,
            PB5<Output<PushPull>>,
            PB6<Output<PushPull>>,
        ),
    >,

    laser: laser::Laser<
        LaserDataBus,
        LaserAlarmBus,
        PA9<Output<PushPull>>,
        PwmChannel<TIM4, 2>,
        PwmChannel<TIM4, 3>,
        PwmChannel<TIM4, 1>,
        PwmChannel<TIM1, 2>,
    >,
}

impl WorkMode<HighPerformanceMode> for HighPerformanceMode {
    fn new(p: cortex_m::Peripherals, dp: stm32f1xx_hal::stm32::Peripherals) -> Self {
        let mut flash = dp.FLASH.constrain();

        let ic = Arc::new(InterruptController::new(p.NVIC));
        let dma_channels = dp.DMA1.split();

        let mut gpioa = dp.GPIOA.split();
        let mut gpiob = dp.GPIOB.split();
        let mut gpioc = dp.GPIOC.split();

        let mut afio = dp.AFIO.constrain();
        let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let laser_power_bus = LaserDataBus(
            gpioa.pa0.into_push_pull_output(&mut gpioa.crl),
            gpioa.pa1.into_push_pull_output(&mut gpioa.crl),
            gpioa.pa2.into_push_pull_output(&mut gpioa.crl),
            gpioa.pa3.into_push_pull_output(&mut gpioa.crl),
            gpioa.pa4.into_push_pull_output(&mut gpioa.crl),
            gpioa.pa5.into_push_pull_output(&mut gpioa.crl),
            gpioa.pa6.into_push_pull_output(&mut gpioa.crl),
            gpioa.pa7.into_push_pull_output(&mut gpioa.crl),
        );

        let laser_alarm_bus = LaserAlarmBus(
            gpioc.pc13.into_floating_input(&mut gpioc.crh),
            gpioc.pc14.into_floating_input(&mut gpioc.crh),
            gpioc.pc15.into_floating_input(&mut gpioc.crh),
        );

        let clocks = HighPerformanceClockConfigProvider::freeze(&mut flash.acr);

        let (l_sync, l_em, l_ee): (
            PwmChannel<TIM4, 1>,
            PwmChannel<TIM4, 2>,
            PwmChannel<TIM4, 3>,
        ) = Timer::new(dp.TIM4, &clocks)
            .pwm_hz(
                (
                    gpiob.pb7.into_alternate_push_pull(&mut gpiob.crl),
                    gpiob.pb8.into_alternate_push_pull(&mut gpiob.crh),
                    gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh),
                ),
                &mut afio.mapr,
                Hertz::kHz(crate::config::LASER_SYNC_CLOCK_KHZ),
            )
            .split();

        let laser_red_beam_pwm = Timer::new(dp.TIM1, &clocks)
            .pwm_hz(
                gpioa.pa10.into_alternate_push_pull(&mut gpioa.crh),
                &mut afio.mapr,
                Hertz::kHz(crate::config::LASER_RED_FREQ_KHZ),
            )
            .split();

        HighPerformanceMode {
            clocks,

            flash: unsafe { Arc::new(Mutex::new(flash).unwrap_unchecked()) },
            crc: unsafe {
                Arc::new(Mutex::new(super::configure_crc_module(dp.CRC.new())).unwrap_unchecked())
            },

            usb: dp.USB,
            usb_dm: gpioa.pa11,
            usb_dp: gpioa.pa12,
            usb_pull_up: pa15.into_push_pull_output_with_state(
                &mut gpioa.crh,
                if !crate::config::USB_PULLUP_ACTVE_LEVEL {
                    stm32f1xx_hal::gpio::PinState::High
                } else {
                    stm32f1xx_hal::gpio::PinState::Low
                },
            ),

            interrupt_controller: ic,

            galvo_ctrl: xy2_100::XY2_100::new(
                dp.TIM2,
                dma_channels.2,
                GPIOB::ptr(),
                (
                    pb3.into_push_pull_output(&mut gpiob.crl),
                    pb4.into_push_pull_output(&mut gpiob.crl),
                    gpiob.pb5.into_push_pull_output(&mut gpiob.crl),
                    gpiob.pb6.into_push_pull_output(&mut gpiob.crl),
                ),
            ),

            laser: laser::Laser::new(
                laser_power_bus,
                Some(gpioa.pa9.into_push_pull_output(&mut gpioa.crh)),
                laser_alarm_bus,
                l_em,
                l_ee,
                l_sync,
                laser_red_beam_pwm,
            ),
        }
    }

    fn ini_static(&mut self) {}

    fn configure_clock(&mut self) {
        crate::time_base::master_counter::MasterCounter::init(
            HighPerformanceClockConfigProvider::master_counter_frequency(),
            self.interrupt_controller.clone(),
        );
    }

    fn start_threads(self) -> Result<(), freertos_rust::FreeRtosError> {
        use crate::gcode::GCode;
        use crate::threads::usbd::Usbd;

        let sys_clk = self.clocks.hclk();
        let tim_ref_clk = self.clocks.pclk1();

        // --------------------------------------------------------------------
        defmt::trace!("Creating usb thread...");
        let usbperith = threads::usbd::UsbdPeriph {
            usb: self.usb,
            pin_dp: self.usb_dp,
            pin_dm: self.usb_dm,
            usb_pull_up: self.usb_pull_up,
        };
        let ic = self.interrupt_controller.clone();
        Usbd::init(
            usbperith,
            ic,
            crate::config::USB_INTERRUPT_PRIO,
            crate::config::USB_PULLUP_ACTVE_LEVEL.into(),
        );

        // --------------------------------------------------------------------

        let (gcode_queue, req_queue) = unsafe {
            (
                Arc::new(freertos_rust::Queue::<GCode>::new(3).unwrap_unchecked()),
                Arc::new(freertos_rust::Queue::<Request>::new(3).unwrap_unchecked()),
            )
        };

        // --------------------------------------------------------------------

        {
            let serial = Usbd::serial_port();
            let gcode_queue_out = gcode_queue.clone();
            let req_queue_out = req_queue.clone();
            let laser = self.laser;

            let mut galvo_ctrl = self.galvo_ctrl;
            galvo_ctrl.begin(self.interrupt_controller.clone(), tim_ref_clk);
            galvo_ctrl.set_pos(0, 0);

            Task::new()
                .name("Motiond")
                .stack_size(
                    (crate::config::MOTION_TASK_STACK_SIZE / core::mem::size_of::<u32>()) as u16,
                )
                .priority(TaskPriority(crate::config::MOTIOND_TASK_PRIO))
                .start(move |_| {
                    threads::motion::motion(
                        serial,
                        gcode_queue_out,
                        req_queue_out,
                        laser,
                        galvo_ctrl,
                        tim_ref_clk,
                    )
                })?;
        }

        // --------------------------------------------------------------------

        {
            let serial = Usbd::serial_port();
            let gcode_srv = {
                defmt::trace!("Creating G-Code server thread...");
                Task::new()
                    .name("G-Code")
                    .stack_size(
                        (crate::config::G_CODE_TASK_STACK_SIZE / core::mem::size_of::<u32>())
                            as u16,
                    )
                    .priority(TaskPriority(crate::config::GCODE_TASK_PRIO))
                    .start(move |_| {
                        threads::gcode_server::gcode_server(serial, gcode_queue, req_queue)
                    })
                    .expect("expect5")
            };
            Usbd::subscribe(gcode_srv);
        }

        // --------------------------------------------------------------------

        let _ = Usbd::strat(
            usb_device::prelude::UsbVidPid(0x0483, 0x573E),
            "OPAL-rust",
            "SCTBElpa",
            "0",
            crate::config::USBD_TASK_STACK_SIZE,
            TaskPriority(crate::config::USBD_TASK_PRIO),
        );

        // --------------------------------------------------------------------

        crate::workmodes::common::create_monitor(sys_clk)?;

        Ok(())
    }

    fn print_clock_config(&self) {
        super::common::print_clock_config(&self.clocks);
    }

    fn flash(&mut self) -> Arc<Mutex<stm32f1xx_hal::flash::Parts>> {
        self.flash.clone()
    }

    fn crc(&mut self) -> Arc<Mutex<stm32f1xx_hal::crc::Crc>> {
        self.crc.clone()
    }
}
