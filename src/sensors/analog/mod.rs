use cortex_m::prelude::_embedded_hal_adc_OneShot;
use freertos_rust::{Duration, Timer};
use stm32l4xx_hal::adc::{Channel, ADC};

use crate::threads::sensor_processor::AChannel;

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
        let timer = Timer::new(Duration::ticks(analog_ticks))
            .set_name(ch.into())
            .set_auto_reload(false)
            .create(move |_| f())
            .unwrap();

        timer.stop(Duration::infinite()).unwrap();

        Self {
            timer,
            adc_ch,
            period: analog_ticks,
        }
    }
}

impl<ADCCH: Channel> AController for AnalogChannel<ADCCH> {
    fn init_cycle(&mut self) {
        self.timer.start(Duration::infinite()).unwrap();
    }

    fn stop(&mut self) {
        self.timer.stop(Duration::infinite()).unwrap();
    }

    fn set_period(&mut self, ticks: u32) {
        self.timer
            .change_period(Duration::infinite(), Duration::ticks(ticks))
            .unwrap();
        self.period = ticks;
    }

    fn period(&self) -> u32 {
        self.period
    }

    fn read(&mut self, adc: &mut ADC) -> u16 {
        adc.read(&mut self.adc_ch).unwrap()
    }
}
