use alloc::sync::Arc;
use freertos_rust::{Duration, Mutex, Task, TaskPriority};

#[allow(unused_imports)]
use stm32l4xx_hal::gpio::{
    Alternate, Analog, Output, PushPull, Speed, PA0, PA1, PA11, PA12, PA2, PA3, PA6, PA7, PA8, PB0,
    PB14, PC10, PD10, PD11, PD13, PE12,
};
use stm32l4xx_hal::{prelude::*, rcc::PllConfig, stm32, time::Hertz};

use crate::control::{laser, xy2_100};
use crate::gcode::Request;
use crate::support::{interrupt_controller::IInterruptController, InterruptController};
use crate::threads;
use crate::workmodes::common::ClockConfigProvider;

use super::WorkMode;

mod clock_config_72;
use clock_config_72::{APB1_DEVIDER, APB2_DEVIDER, PLL_CFG, SAI_DIVIDER, SAI_MULTIPLIER};

struct HighPerformanceClockConfigProvider;

impl ClockConfigProvider for HighPerformanceClockConfigProvider {
    fn core_frequency() -> Hertz {
        let f = crate::config::XTAL_FREQ * PLL_CFG.1 / (PLL_CFG.0 * PLL_CFG.2);
        Hertz::Hz(f)
    }

    fn apb1_frequency() -> Hertz {
        Hertz::Hz(Self::core_frequency().to_Hz() / APB1_DEVIDER)
    }

    fn apb2_frequency() -> Hertz {
        Hertz::Hz(Self::core_frequency().to_Hz() / APB2_DEVIDER)
    }

    // stm32_cube: if APB devider > 1, timers freq APB*2
    fn master_counter_frequency() -> Hertz {
        if APB1_DEVIDER > 1 {
            Hertz::Hz(Self::apb1_frequency().to_Hz() * 2)
        } else {
            Self::apb1_frequency()
        }
    }

    fn pll_config() -> PllConfig {
        PllConfig::new(
            PLL_CFG.0 as u8,
            PLL_CFG.1 as u8,
            crate::workmodes::common::to_pll_devider(PLL_CFG.2),
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

    led_pin: PC10<Output<PushPull>>,

    galvo_ctrl: xy2_100::XY2_100<PA2<Output<PushPull>>>,

    laser_pwm_pin: PA2<Alternate<PushPull, 14>>, // CLK
    laser_enable_pin: PA8<Output<PushPull>>,     //F_PRES
    tim15: stm32::TIM15,
}

impl WorkMode<HighPerformanceMode> for HighPerformanceMode {
    fn new(p: cortex_m::Peripherals, dp: stm32l4xx_hal::stm32l4::stm32l4x3::Peripherals) -> Self {
        let mut rcc = dp.RCC.constrain();
        let ic = Arc::new(InterruptController::new(p.NVIC));
        let dma_channels = dp.DMA2.split(&mut rcc.ahb1);

        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
        let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);

        let gpiod = dp.GPIOD.split(&mut rcc.ahb2);

        /*
        let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
        let mut gpioe = dp.GPIOE.split(&mut rcc.ahb2);
        */

        HighPerformanceMode {
            flash: unsafe { Arc::new(Mutex::new(dp.FLASH.constrain()).unwrap_unchecked()) },
            crc: unsafe {
                Arc::new(
                    Mutex::new(super::configure_crc_module(dp.CRC.constrain(&mut rcc.ahb1)))
                        .unwrap_unchecked(),
                )
            },

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

            rcc,

            led_pin: gpioc
                .pc10
                .into_push_pull_output_in_state(&mut gpioc.moder, &mut gpioc.otyper, PinState::High)
                .set_speed(Speed::Low),

            // PCB
            // верхняя флешка
            // via: *  *  *  *  *  *
            // chk: A3 D6 D3 D5 D7 D11
            // нижняя флешка
            // via: *  *  *  *  *  *
            // chk: A3 x  A2 B0 x  D11
            galvo_ctrl: xy2_100::XY2_100::new(dp.TIM7, gpiod, dma_channels.5, 6, 3, 5, 7, None),

            // pwm and enable pins
            laser_pwm_pin: gpioa
                .pa2
                .into_alternate(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl)
                .set_speed(Speed::Medium),
            laser_enable_pin: gpioa
                .pa8
                .into_push_pull_output_in_state(&mut gpioa.moder, &mut gpioa.otyper, PinState::Low)
                .set_speed(Speed::Low),

            tim15: dp.TIM15,
        }
    }

    fn ini_static(&mut self) {}

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
                    .bits(SAI_MULTIPLIER)
                    .pllsai1q()
                    .bits(SAI_DIVIDER)
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
                    Hertz::Hz(crate::config::XTAL_FREQ), // onboard crystall
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
        crate::time_base::master_counter::MasterCounter::init(
            HighPerformanceClockConfigProvider::master_counter_frequency(),
            self.interrupt_controller.clone(),
        );

        self.clocks = Some(clocks);
    }

    fn start_threads(mut self) -> Result<(), freertos_rust::FreeRtosError> {
        use crate::gcode::GCode;
        use crate::threads::usbd::Usbd;

        let sys_clk = unsafe { self.clocks.unwrap_unchecked().hclk() };
        let tim_ref_clk = unsafe { self.clocks.unwrap_unchecked().pclk1() };

        crate::support::led::led_init(self.led_pin);

        // --------------------------------------------------------------------
        defmt::trace!("Creating usb thread...");
        let usbperith = threads::usbd::UsbdPeriph {
            usb: self.usb,
            pin_dp: self.usb_dp,
            pin_dm: self.usb_dm,
        };
        let ic = self.interrupt_controller.clone();
        Usbd::init(usbperith, ic, crate::config::USB_INTERRUPT_PRIO);

        // --------------------------------------------------------------------

        let (gcode_queue, req_queue) = unsafe {
            (
                Arc::new(freertos_rust::Queue::<GCode>::new(3).unwrap_unchecked()),
                Arc::new(freertos_rust::Queue::<Request>::new(3).unwrap_unchecked()),
            )
        };

        let mut galvo_ctrl = self.galvo_ctrl;
        galvo_ctrl.begin(self.interrupt_controller.clone(), tim_ref_clk);

        galvo_ctrl.set_pos(0, 0);

        let pwm_pin = self.tim15.pwm(
            self.laser_pwm_pin,
            1.kHz(),
            unsafe { *self.clocks.as_ref().unwrap_unchecked() },
            &mut self.rcc.apb2,
        );

        let laser = laser::Laser::new(pwm_pin, self.laser_enable_pin);

        // --------------------------------------------------------------------

        {
            let serial = Usbd::serial_port();
            let gcode_queue_out = gcode_queue.clone();
            let req_queue_out = req_queue.clone();

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
            Usbd::subsbrbe(gcode_srv);
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
        super::common::print_clock_config(&self.clocks, "HSI48");
    }

    fn flash(&mut self) -> Arc<Mutex<stm32l4xx_hal::flash::Parts>> {
        self.flash.clone()
    }

    fn crc(&mut self) -> Arc<Mutex<stm32l4xx_hal::crc::Crc>> {
        self.crc.clone()
    }
}
