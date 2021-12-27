use cortex_m::prelude::_embedded_hal_adc_OneShot;
use freertos_rust::{Duration, Timer};
use stm32l4xx_hal::adc::{Channel, ADC};

use crate::{support::new_freertos_timer, threads::sensor_processor::AChannel};

pub trait AController {
    fn init_cycle(&mut self);
    fn stop(&mut self);
    fn set_period(&mut self, ticks: u32);
    fn period(&self) -> u32;

    fn read(&mut self, adc: &mut ADC) -> u16;
}

pub struct AnalogChannel<ADCCH: Channel> {
    timer: Timer,
    adc_ch: ADCCH,
    period: u32,
}

impl<ADCCH: Channel> AnalogChannel<ADCCH> {
    pub fn new<F>(ch: AChannel, adc_ch: ADCCH, analog_ticks: u32, f: F) -> Self
    where
        F: Fn() + Send + 'static,
        ADCCH: Send,
    {
        let timer = new_freertos_timer(Duration::ticks(analog_ticks), ch.into(), f);
        let _ = timer.stop(Duration::infinite());

        Self {
            timer,
            adc_ch,
            period: analog_ticks,
        }
    }
}

impl<ADCCH: Channel> AController for AnalogChannel<ADCCH> {
    fn init_cycle(&mut self) {
        let _ = self.timer.start(Duration::infinite());
    }

    fn stop(&mut self) {
        let _ = self.timer.stop(Duration::infinite());
    }

    fn set_period(&mut self, ticks: u32) {
        let _ = self
            .timer
            .change_period(Duration::infinite(), Duration::ticks(ticks));
        self.period = ticks;
    }

    fn period(&self) -> u32 {
        self.period
    }

    fn read(&mut self, adc: &mut ADC) -> u16 {
        adc.read(&mut self.adc_ch).unwrap_or_default()
    }
}
