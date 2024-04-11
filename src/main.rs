#![no_main]
#![no_std]
#![feature(macro_metavar_expr)]

mod config;
mod control;
mod gcode;
mod hw;
mod support;

use panic_abort as _;
use rtic::app;

use stm32f1xx_hal::afio::AfioExt;
use stm32f1xx_hal::dma::DmaExt;
use stm32f1xx_hal::flash::FlashExt;
use stm32f1xx_hal::gpio::{
    Floating, GpioExt, Input, Output, PushPull, PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA9, PB3,
    PB4, PB5, PB6, PC13, PC14, PC15,
};
use stm32f1xx_hal::rcc::{HPre, PPre};
use stm32f1xx_hal::time::Hertz;
use stm32f1xx_hal::timer::{PwmChannel, Timer};
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};

use stm32f1xx_hal::dma::dma1;
use stm32f1xx_hal::pac::{Interrupt, TIM1, TIM2, TIM4};

use usb_device::prelude::{UsbDevice, UsbDeviceBuilder};

use usbd_serial::SerialPort;

use systick_monotonic::Systick;

use support::clocking::{ClockConfigProvider, MyConfig};

use control::xy2_100::XY2_100Interface;

use hw::{ADC_DEVIDER, AHB_DEVIDER, APB1_DEVIDER, APB2_DEVIDER, PLL_MUL, PLL_P_DIV, USB_DEVIDER};

//-----------------------------------------------------------------------------

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
        // TIM3 - APB1
        if APB2_DEVIDER == PPre::DIV1 {
            Self::core_frequency()
        } else {
            Self::core_frequency() * 2
        }
    }

    fn xtal2master_freq_multiplier() -> f32 {
        PLL_MUL as f32
            / (Self::ppl_div2val(PLL_P_DIV)
                * Self::ahb_dev2val(AHB_DEVIDER)
                * Self::apb_dev2val(APB2_DEVIDER)) as f32
    }

    fn to_config() -> MyConfig {
        MyConfig {
            hse_p_div: PLL_P_DIV,
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

type Galvo = control::xy2_100::XY2_100<
    TIM2,
    dma1::C2,
    (
        PB3<Output<PushPull>>,
        PB4<Output<PushPull>>,
        PB5<Output<PushPull>>,
        PB6<Output<PushPull>>,
    ),
>;

#[app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [RTCALARM])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_device: UsbDevice<'static, UsbBusType>,
        serial: SerialPort<'static, UsbBus<Peripheral>>,
        gcode_queue: heapless::Deque<gcode::GCode, { config::GCODE_QUEUE_SIZE }>,
        request_queue: heapless::Deque<gcode::Request, { config::GCODE_QUEUE_SIZE }>,
    }

    #[local]
    struct Local {
        motion_mgr: gcode::MotionMGR<
            control::laser::Laser<
                LaserDataBus,
                LaserAlarmBus,
                PA9<Output<PushPull>>,
                PwmChannel<TIM4, 2>,
                PwmChannel<TIM4, 3>,
                PwmChannel<TIM4, 1>,
                PwmChannel<TIM1, 2>,
            >,
            Galvo,
        >,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<{ config::SYSTICK_RATE_HZ }>;

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        static mut USB_BUS: Option<usb_device::bus::UsbBusAllocator<UsbBusType>> = None;

        let mut flash = ctx.device.FLASH.constrain();

        let dma_channels = ctx.device.DMA1.split();

        let mut gpioa = ctx.device.GPIOA.split();
        let mut gpiob = ctx.device.GPIOB.split();
        let mut gpioc = ctx.device.GPIOC.split();

        let mut afio = ctx.device.AFIO.constrain();
        let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let mut usb_pull_up = pa15.into_push_pull_output_with_state(
            &mut gpioa.crh,
            if !config::USB_PULLUP_ACTVE_LEVEL {
                stm32f1xx_hal::gpio::PinState::High
            } else {
                stm32f1xx_hal::gpio::PinState::Low
            },
        );

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

        let mono = Systick::new(ctx.core.SYST, clocks.sysclk().to_Hz());

        let laser_pwm_tim_clocks = clocks.pclk1_tim();
        let (l_sync, l_em, l_ee) = Timer::new(ctx.device.TIM4, &clocks)
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

        let laser_red_beam_pwm = Timer::new(ctx.device.TIM1, &clocks)
            .pwm_hz(
                gpioa.pa10.into_alternate_push_pull(&mut gpioa.crh),
                &mut afio.mapr,
                Hertz::kHz(crate::config::LASER_RED_FREQ_KHZ),
            )
            .split();

        let usb = Peripheral {
            usb: ctx.device.USB,
            pin_dm: gpioa.pa11,
            pin_dp: gpioa.pa12,
        };

        unsafe {
            USB_BUS.replace(UsbBus::new(usb));
        }

        let serial = SerialPort::new(unsafe { USB_BUS.as_ref().unwrap_unchecked() });

        let usb_dev = UsbDeviceBuilder::new(
            unsafe { USB_BUS.as_ref().unwrap_unchecked() },
            usb_device::prelude::UsbVidPid(0x16c0, 0x27dd),
        )
        .manufacturer("SCTBElpa")
        .product("OPAL-rust")
        .serial_number("1")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

        //---------------------------------------------------------------------

        usb_pull_up.toggle(); // enable USB

        //---------------------------------------------------------------------

        let mut galvo_ctrl = control::xy2_100::XY2_100::new(
            ctx.device.TIM2,
            dma_channels.2,
            stm32f1xx_hal::pac::GPIOB::ptr(),
            (
                pb3.into_push_pull_output(&mut gpiob.crl),
                pb4.into_push_pull_output(&mut gpiob.crl),
                gpiob.pb5.into_push_pull_output(&mut gpiob.crl),
                gpiob.pb6.into_push_pull_output(&mut gpiob.crl),
            ),
        );

        galvo_ctrl.begin(clocks.pclk1_tim());

        let laser = control::laser::Laser::new(
            laser_power_bus,
            Some(gpioa.pa9.into_push_pull_output(&mut gpioa.crh)),
            laser_alarm_bus,
            l_em,
            l_ee,
            l_sync,
            laser_red_beam_pwm,
            laser_pwm_tim_clocks,
        );

        let mut motion_mgr = gcode::MotionMGR::new(galvo_ctrl, laser, config::GCODE_QUEUE_SIZE);

        motion_mgr.begin();

        //---------------------------------------------------------------------

        (
            Shared {
                usb_device: usb_dev,
                serial,
                gcode_queue: heapless::Deque::new(),
                request_queue: heapless::Deque::new(),
            },
            Local { motion_mgr },
            init::Monotonics(mono),
        )
    }

    //-------------------------------------------------------------------------

    #[task(binds = USB_HP_CAN_TX, shared = [usb_device, serial, gcode_queue, request_queue], priority = 1)]
    fn usb_tx(ctx: usb_tx::Context) {
        let mut usb_device = ctx.shared.usb_device;
        let mut serial = ctx.shared.serial;
        let mut gcode_queue = ctx.shared.gcode_queue;
        let mut request_queue = ctx.shared.request_queue;

        let gcode_pusher = move |gcode| gcode_queue.lock(|q| q.push_back(gcode));
        let request_pusher = move |request| request_queue.lock(|q| q.push_back(request));

        if !(&mut usb_device, &mut serial).lock(move |usb_device, serial| {
            super::usb_poll(usb_device, serial, gcode_pusher, request_pusher)
        }) {
            cortex_m::peripheral::NVIC::mask(Interrupt::USB_HP_CAN_TX);
            cortex_m::peripheral::NVIC::mask(Interrupt::USB_LP_CAN_RX0);
        }
    }

    #[task(binds = USB_LP_CAN_RX0, shared = [usb_device, serial, gcode_queue, request_queue], priority = 1)]
    fn usb_rx0(ctx: usb_rx0::Context) {
        let mut usb_device = ctx.shared.usb_device;
        let mut serial = ctx.shared.serial;
        let mut gcode_queue = ctx.shared.gcode_queue;
        let mut request_queue = ctx.shared.request_queue;

        let gcode_pusher = move |gcode| gcode_queue.lock(|q| q.push_back(gcode));
        let request_pusher = move |request| request_queue.lock(|q| q.push_back(request));

        if !(&mut usb_device, &mut serial).lock(move |usb_device, serial| {
            super::usb_poll(usb_device, serial, gcode_pusher, request_pusher)
        }) {
            cortex_m::peripheral::NVIC::mask(Interrupt::USB_HP_CAN_TX);
            cortex_m::peripheral::NVIC::mask(Interrupt::USB_LP_CAN_RX0);
        }
    }

    #[task(binds = DMA1_CHANNEL2, priority = 2)]
    fn dma1_ch2(_ctx: dma1_ch2::Context) {
        unsafe {
            Galvo::dma_event();
        }
    }

    //-------------------------------------------------------------------------

    #[idle(shared=[gcode_queue, request_queue, serial], local = [motion_mgr])]
    fn idle(ctx: idle::Context) -> ! {
        use core::str::FromStr;

        const NANOSEC_PER_SYSTICK: u64 = 1_000_000_000u64 / config::SYSTICK_RATE_HZ as u64;

        let mut gcode_queue = ctx.shared.gcode_queue;
        let mut request_queue = ctx.shared.request_queue;
        let mut serial = ctx.shared.serial;

        let mm = ctx.local.motion_mgr;
        //let mut mm = ctx.shared.motion_mgr;

        fn send<const N: usize>(
            serial: &mut shared_resources::serial_that_needs_to_be_locked,
            msg: Option<heapless::String<N>>,
        ) {
            if let Some(msg) = msg {
                let _ = serial.lock(|s| s.write(msg.as_bytes()));
                //rtic::pend(stm32f1xx_hal::device::Interrupt::USB_HP_CAN_TX);
            }
        }

        loop {
            let res = if mm.tic(monotonics::MonoTimer::now().ticks() * NANOSEC_PER_SYSTICK)
                == gcode::MotionStatus::IDLE
            {
                let (msg, avlb) =
                    gcode_queue.lock(|gcq| (gcq.pop_front(), gcq.capacity() - gcq.len()));
                if let Some(mut gcode) = msg {
                    unsafe {
                        cortex_m::peripheral::NVIC::unmask(Interrupt::USB_HP_CAN_TX);
                        cortex_m::peripheral::NVIC::unmask(Interrupt::USB_LP_CAN_RX0);
                    }
                    match mm.process(&mut gcode, avlb - 1) {
                        Ok(Some(s)) => Some(s),
                        Ok(None) => {
                            Some(unsafe { config::HlString::from_str("ok\n\r").unwrap_unchecked() })
                        }
                        Err(s) => Some(s),
                    }
                } else {
                    None
                }
            } else {
                None
            };

            send(&mut serial, res);

            let res = if let Some(req) = request_queue.lock(|rq| rq.pop_front()) {
                Some(mm.process_status_req(&req))
            } else {
                None
            };

            match res {
                Some(Ok(msg)) => {
                    send(&mut serial, msg);
                }
                Some(Err(msg)) => {
                    send(&mut serial, Some(msg));
                }
                _ => {}
            }

            if !mm.is_busy() {
                cortex_m::asm::wfi();
            }
        }
    }
}

fn usb_poll<B: usb_device::bus::UsbBus, GP, RP>(
    usb_dev: &mut usb_device::prelude::UsbDevice<'static, B>,
    serial: &mut usbd_serial::SerialPort<'static, B>,
    gcode_pusher: GP,
    request_pusher: RP,
) -> bool
where
    GP: FnMut(gcode::GCode) -> Result<(), gcode::GCode>,
    RP: FnMut(gcode::Request) -> Result<(), gcode::Request>,
    B: usb_device::bus::UsbBus,
{
    use gcode::SerialErrResult;
    use heapless::String;

    static mut BUF: String<{ gcode::MAX_LEN }> = String::new();

    if !usb_dev.poll(&mut [serial]) {
        return true;
    }

    let trimm_buff = |trimm_size| unsafe {
        if trimm_size > 0 {
            if trimm_size == BUF.len() {
                BUF.clear()
            } else {
                let new_buf = BUF.chars().skip(trimm_size).collect();
                BUF = new_buf;
            }
        }
    };

    match gcode::serial_process(
        serial,
        unsafe { &mut *core::ptr::addr_of_mut!(BUF) },
        gcode_pusher,
        request_pusher,
    ) {
        Ok(trimm_size) => trimm_buff(trimm_size),
        Err(SerialErrResult::OutOfMemory) => {
            unsafe { BUF.clear() };
            serial.write(b"Command too long! (150)").unwrap();
        }
        Err(SerialErrResult::Incomplead(_trimm_size)) => {
            //serial.write(b"Command buffer full").unwrap();
            return false;
        }
        _ => {}
    }

    true
}
