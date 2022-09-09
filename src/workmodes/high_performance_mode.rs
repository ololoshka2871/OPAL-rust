use alloc::sync::Arc;

use freertos_rust::{Duration, Mutex, Task, TaskPriority};

use stm32f1xx_hal::{
    flash,
    gpio::{
        Alternate, Analog, Floating, Input, Output, PinState, PushPull, PA0, PA1, PA10, PA11, PA12,
        PA15, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PB0, PB14, PB7, PB8, PB9, PC10, PC13, PC14,
        PC15, PD10, PD11, PD13, PE12,
    },
    prelude::*,
    rcc::{HPre, PPre},
    stm32,
    time::Hertz,
};

use crate::control::{laser, xy2_100};
use crate::gcode::Request;
use crate::support::{interrupt_controller::IInterruptController, InterruptController};
use crate::threads;
use crate::workmodes::common::ClockConfigProvider;

use super::WorkMode;

mod clock_config_72;
use clock_config_72::{ADC_DEVIDER, AHB_DEVIDER, APB1_DEVIDER, APB2_DEVIDER, PLL_MUL, USB_DEVIDER};

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
}

impl ClockConfigProvider for HighPerformanceClockConfigProvider {
    fn core_frequency() -> Hertz {
        let f = crate::config::XTAL_FREQ * PLL_MUL / Self::ahb_dev2val(AHB_DEVIDER);
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
        PLL_MUL as f32 / (Self::ahb_dev2val(AHB_DEVIDER) * Self::apb_dev2val(APB2_DEVIDER)) as f32
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

#[allow(unused)]
pub struct HighPerformanceMode {
    flash: Arc<Mutex<stm32f1xx_hal::flash::Parts>>,

    clocks: stm32f1xx_hal::rcc::Clocks,

    usb: stm32f1xx_hal::stm32::USB,
    usb_dm: PA11<Input<Floating>>,
    usb_dp: PA12<Input<Floating>>,
    usb_pull_up: PA15<Output<PushPull>>,

    interrupt_controller: Arc<dyn IInterruptController>,

    crc: Arc<Mutex<stm32f1xx_hal::crc::Crc>>,

    galvo_ctrl: xy2_100::XY2_100<PA2<Output<PushPull>>>,

    laser_timer: stm32::TIM4,
    laser_red_bam_timer: stm32::TIM1,
    laser_power_bus: (
        PA0<Output<PushPull>>,
        PA1<Output<PushPull>>,
        PA2<Output<PushPull>>,
        PA3<Output<PushPull>>,
        PA4<Output<PushPull>>,
        PA5<Output<PushPull>>,
        PA6<Output<PushPull>>,
        PA7<Output<PushPull>>,
    ),
    laser_power_latch: PA9<Output<PushPull>>,
    laser_status: (
        PC13<Input<Floating>>,
        PC14<Input<Floating>>,
        PC15<Input<Floating>>,
    ),
    laser_emission_modulatiuon: PB8<Alternate<PushPull>>,
    laser_emission_enable: PB9<Alternate<PushPull>>,
    laser_sync: PB7<Alternate<PushPull>>,
    laser_red_bam: PA10<Alternate<PushPull>>,
}

impl WorkMode<HighPerformanceMode> for HighPerformanceMode {
    fn new(p: cortex_m::Peripherals, dp: stm32f1xx_hal::stm32::Peripherals) -> Self {
        let rcc = dp.RCC.constrain();
        let mut flash = dp.FLASH.constrain();

        let ic = Arc::new(InterruptController::new(p.NVIC));
        let dma_channels = dp.DMA2.split();

        let mut gpioa = dp.GPIOA.split();
        let mut gpiob = dp.GPIOB.split();
        let mut gpioc = dp.GPIOC.split();

        let mut afio = dp.AFIO.constrain();
        let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        HighPerformanceMode {
            clocks: rcc.cfgr.freeze_with_config(
                HighPerformanceClockConfigProvider::to_config(),
                &mut flash.acr,
            ),

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
                &mut gpiob,
                dma_channels.2,
                3,
                4,
                5,
                6,
            ),

            // pwm and enable pins
            laser_timer: dp.TIM4,
            laser_red_bam_timer: dp.TIM1,
            laser_power_bus: (
                gpioa.pa0.into_push_pull_output(&mut gpioa.crl),
                gpioa.pa1.into_push_pull_output(&mut gpioa.crl),
                gpioa.pa2.into_push_pull_output(&mut gpioa.crl),
                gpioa.pa3.into_push_pull_output(&mut gpioa.crl),
                gpioa.pa4.into_push_pull_output(&mut gpioa.crl),
                gpioa.pa5.into_push_pull_output(&mut gpioa.crl),
                gpioa.pa6.into_push_pull_output(&mut gpioa.crl),
                gpioa.pa7.into_push_pull_output(&mut gpioa.crl),
            ),
            laser_power_latch: gpioa.pa9.into_push_pull_output(&mut gpioa.crh),
            laser_status: (
                gpioc.pc13.into_floating_input(&mut gpioc.crh),
                gpioc.pc14.into_floating_input(&mut gpioc.crh),
                gpioc.pc15.into_floating_input(&mut gpioc.crh),
            ),
            laser_emission_modulatiuon: todo!(),
            laser_emission_enable: todo!(),
            laser_sync: todo!(),
            laser_red_bam: todo!(),
        }
    }

    fn ini_static(&mut self) {}

    fn configure_clock(&mut self) {
        crate::time_base::master_counter::MasterCounter::init(
            HighPerformanceClockConfigProvider::master_counter_frequency(),
            self.interrupt_controller.clone(),
        );
    }

    fn start_threads(mut self) -> Result<(), freertos_rust::FreeRtosError> {
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

        let mut galvo_ctrl = self.galvo_ctrl;
        galvo_ctrl.begin(self.interrupt_controller.clone(), tim_ref_clk);

        galvo_ctrl.set_pos(0, 0);

        let pwm_pin = self.tim15.pwm(self.laser_pwm_pin, 0, 1.kHz(), &self.clocks);

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
