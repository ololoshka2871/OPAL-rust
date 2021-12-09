use core::fmt::Debug;
use core::marker::PhantomData;

use stm32l4xx_hal::{gpio::State, prelude::OutputPin};

use super::{f_ch_processor::FChProcessor, hw_in_counters::InCounter, TimerEvent};

pub struct FreqmeterController<'a, TIM, DMA, INPIN, ENPIN>
where
    TIM: InCounter<DMA, INPIN>,
{
    freqmeter: &'a mut TIM,
    gpio_pin: ENPIN,
    prev: u32,
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
        //freertos_rust::CurrentTask::delay(freertos_rust::Duration::ms(10));
        self.freqmeter.cold_start();
    }

    fn diasbe(&mut self) {
        self.freqmeter.stop();
        self.set_lvl(crate::config::GENERATOR_DISABLE_LVL);
    }

    fn restart(&mut self) {
        self.freqmeter.cold_start();
    }

    fn set_target(&mut self, new_target: u32) {
        self.freqmeter.stop();
        self.freqmeter.set_target32(new_target);
        self.freqmeter.cold_start();
    }

    fn input_captured(&mut self, event: TimerEvent, captured: u32) -> Option<u32> {
        match event {
            TimerEvent::Start => {
                self.prev = captured;
                None
            }
            TimerEvent::Stop => {
                let diff = if self.prev <= captured {
                    captured - self.prev
                } else {
                    u32::MAX - self.prev + captured
                };
                self.prev = captured;

                Some(diff)
            }
        }
    }
}

impl<'a, TIM, DMA, INPIN, ENPIN> FreqmeterController<'a, TIM, DMA, INPIN, ENPIN>
where
    TIM: InCounter<DMA, INPIN>,
    ENPIN: OutputPin,
    <ENPIN as OutputPin>::Error: Debug,
{
    pub fn new(freqmeter: &'a mut TIM, gpio_pin: ENPIN) -> Self {
        Self {
            freqmeter,
            gpio_pin,
            prev: 0,
            _phantomdata1: PhantomData,
            _phantomdata2: PhantomData,
        }
    }

    fn set_lvl(&mut self, lvl: State) {
        match lvl {
            State::High => self.gpio_pin.set_high().unwrap(),
            State::Low => self.gpio_pin.set_low().unwrap(),
        }
    }
}
