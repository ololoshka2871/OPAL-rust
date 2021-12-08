use core::fmt::Debug;
use core::marker::PhantomData;

use stm32l4xx_hal::{gpio::State, prelude::OutputPin};

use super::{f_ch_processor::FChProcessor, hw_in_counters::InCounter};

pub type FrefGetter<T> = fn() -> Result<f64, T>;

pub struct FreqmeterController<'a, TIM, DMA, INPIN, ENPIN, TSE>
where
    TIM: InCounter<DMA, INPIN>,
{
    freqmeter: &'a mut TIM,
    gpio_pin: ENPIN,
    fref_multiplier: f64,
    ferf_getter: FrefGetter<TSE>,
    prev: u32,
    startup: bool,
    _phantomdata1: PhantomData<DMA>,
    _phantomdata2: PhantomData<INPIN>,
}

impl<'a, TIM, DMA, INPIN, ENPIN, TSE> FChProcessor<TSE>
    for FreqmeterController<'a, TIM, DMA, INPIN, ENPIN, TSE>
where
    TIM: InCounter<DMA, INPIN>,
    ENPIN: OutputPin,
    <ENPIN as OutputPin>::Error: Debug,
{
    fn enable(&mut self) {
        self.set_lvl(crate::config::GENERATOR_ENABLE_LVL);
        self.startup = true;
        self.freqmeter.cold_start();
    }

    fn diasbe(&mut self) {
        self.freqmeter.stop();
        self.set_lvl(crate::config::GENERATOR_DISABLE_LVL);
    }

    fn is_initial_result(&mut self) -> bool {
        if self.startup {
            self.startup = false;
            true
        } else {
            false
        }
    }

    fn adaptate(&mut self) -> Result<u32, ()> {
        todo!()
    }

    fn input_captured(&mut self, captured: u32) -> Option<u32> {
        if self.is_initial_result() {
            None
        } else {
            let diff = if self.prev <= captured {
                captured - self.prev
            } else {
                u32::MAX - self.prev + captured
            };
            self.prev = captured;

            Some(diff)
        }
    }

    fn calc_freq(&mut self, target: u32, diff: u32) -> Result<f64, TSE> {
        let fref = self.fref_multiplier * (self.ferf_getter)()?;
        let f = fref * target as f64 / diff as f64;

        Ok(f)
    }
}

impl<'a, TIM, DMA, INPIN, ENPIN, TSE> FreqmeterController<'a, TIM, DMA, INPIN, ENPIN, TSE>
where
    TIM: InCounter<DMA, INPIN>,
    ENPIN: OutputPin,
    <ENPIN as OutputPin>::Error: Debug,
{
    pub fn new(
        freqmeter: &'a mut TIM,
        gpio_pin: ENPIN,
        fref_multiplier: f64,
        ferf_getter: FrefGetter<TSE>,
    ) -> Self {
        Self {
            freqmeter,
            gpio_pin,
            fref_multiplier,
            ferf_getter,
            prev: 0,
            startup: false,
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
