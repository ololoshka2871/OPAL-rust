use core::fmt::Debug;
use core::marker::PhantomData;

use freertos_rust::{Duration, Timer};
use stm32l4xx_hal::{gpio::PinState, prelude::OutputPin};

use crate::threads::sensor_processor::FChannel;

use super::{f_ch_processor::FChProcessor, hw_in_counters::InCounter, TimerEvent};

pub struct FreqmeterController<'a, TIM, DMA, INPIN, ENPIN>
where
    TIM: InCounter<DMA, INPIN>,
{
    freqmeter: &'a mut TIM,
    gpio_pin: ENPIN,
    prev: u32,
    start: bool,
    no_signal_guard: Timer,
    _phantomdata1: PhantomData<DMA>,
    _phantomdata2: PhantomData<INPIN>,
}

impl<'a, TIM, DMA, INPIN, ENPIN> FChProcessor for FreqmeterController<'a, TIM, DMA, INPIN, ENPIN>
where
    TIM: InCounter<DMA, INPIN>,
    ENPIN: OutputPin,
    <ENPIN as OutputPin>::Error: Debug,
{
    fn enable(&mut self) {
        self.set_lvl(crate::config::GENERATOR_ENABLE_LVL);
        self.start = true;
        //freertos_rust::CurrentTask::delay(freertos_rust::Duration::ms(10));
        self.freqmeter.cold_start();
        self.no_signal_guard.start(Duration::infinite()).unwrap();
    }

    fn diasble(&mut self) {
        self.no_signal_guard.stop(Duration::infinite()).unwrap();
        self.freqmeter.stop();
        self.set_lvl(crate::config::GENERATOR_DISABLE_LVL);
    }

    fn restart(&mut self) {
        self.start = true;
        self.reset_guard();
        self.freqmeter.cold_start();
    }

    fn reset_guard(&mut self) {
        self.no_signal_guard.stop(Duration::infinite()).unwrap();
        self.no_signal_guard.start(Duration::infinite()).unwrap();
    }

    fn set_target(&mut self, new_target: u32, guard_ticks: u32) {
        self.no_signal_guard.stop(Duration::infinite()).unwrap();
        self.freqmeter.stop();

        self.no_signal_guard
            .change_period(Duration::infinite(), Duration::ticks(guard_ticks))
            .unwrap();

        self.freqmeter.set_target32(new_target);
        self.restart();
    }

    #[allow(unused_mut)]
    #[allow(unused_assignments)]
    fn input_captured(&mut self, mut event: TimerEvent, captured: u32) -> Option<u32> {
        #[cfg(not(feature = "freqmeter-start-stop"))]
        {
            event = if self.start {
                self.start = false;
                TimerEvent::Start
            } else {
                TimerEvent::Stop
            };
        }

        match event {
            TimerEvent::Start => {
                self.prev = captured;
                None
            }
            TimerEvent::Stop => {
                let diff = if self.prev <= captured {
                    captured - self.prev
                } else {
                    //defmt::warn!("Master overflow");
                    u32::MAX - self.prev + captured
                };
                self.prev = captured;

                Some(diff)
            }
        }
    }

    fn target(&self) -> u32 {
        self.freqmeter.target32()
    }
}

impl<'a, TIM, DMA, INPIN, ENPIN> FreqmeterController<'a, TIM, DMA, INPIN, ENPIN>
where
    TIM: InCounter<DMA, INPIN>,
    ENPIN: OutputPin,
    <ENPIN as OutputPin>::Error: Debug,
{
    pub fn new<F>(
        freqmeter: &'a mut TIM,
        gpio_pin: ENPIN,
        ch: FChannel,
        initial_guard_ticks: u32,
        f: F,
    ) -> Self
    where
        F: Fn() + Send + 'static,
    {
        let timer = Timer::new(Duration::ticks(initial_guard_ticks))
            .set_name(ch.into())
            .set_auto_reload(false)
            .create(move |_| f())
            .unwrap();

        timer.stop(Duration::infinite()).unwrap();

        Self {
            freqmeter,
            gpio_pin,
            prev: 0,
            start: false,
            no_signal_guard: timer,
            _phantomdata1: PhantomData,
            _phantomdata2: PhantomData,
        }
    }

    fn set_lvl(&mut self, lvl: PinState) {
        match lvl {
            PinState::High => self.gpio_pin.set_high().unwrap(),
            PinState::Low => self.gpio_pin.set_low().unwrap(),
        }
    }
}
