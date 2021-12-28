use core::ops::DerefMut;

use alloc::sync::Arc;
use freertos_rust::{Duration, Mutex};
use stm32l4xx_hal::{adc::ADC, time::Hertz};

use crate::{
    sensors::analog::AController,
    threads::sensor_processor::{AChannel, FChannel},
    workmodes::output_storage::OutputStorage,
};

use super::RawValueProcessor;

pub struct HighPerformanceProcessor {
    output: Arc<Mutex<OutputStorage>>,
    fref_multiplier: f64,
    sysclk: Hertz,
}

impl HighPerformanceProcessor {
    pub fn new(output: Arc<Mutex<OutputStorage>>, fref_multiplier: f64, sysclk: Hertz) -> Self {
        Self {
            output,
            fref_multiplier,
            sysclk,
        }
    }
}

impl RawValueProcessor for HighPerformanceProcessor {
    fn process_f_result(
        &mut self,
        ch: FChannel,
        target: u32,
        result: u32,
    ) -> (bool, Option<(u32, u32)>) {
        let config = super::channel_config(ch);
        let mut new_target_opt = None;
        if config.enabled {
            if let Ok(mut guard) = self.output.lock(Duration::infinite()) {
                guard.targets[ch as usize] = target;
                guard.results[ch as usize] = Some(result);
            }

            let f = super::calc_freq(self.fref_multiplier, target, result);

            if let Ok(mut guard) = self.output.lock(Duration::infinite()) {
                guard.frequencys[ch as usize] = Some(f);
            }

            let _ = self.output.lock(Duration::infinite()).map(|mut g| {
                let o = g.deref_mut();
                match ch {
                    FChannel::Pressure => super::calc_pressure(f, o),
                    FChannel::Temperature => super::calc_temperature(f, o),
                }
            });

            let (new_target, guard_ticks) = super::calc_new_target(ch, f, &self.sysclk);

            if super::abs_difference(new_target, target)
                > crate::config::MINIMUM_ADAPTATION_INTERVAL
            {
                defmt::warn!(
                    "Adaptation ch. {}, new target {}, guard: {} ticks",
                    ch,
                    new_target,
                    guard_ticks
                );
                new_target_opt = Some((new_target, guard_ticks));
            }

            (true, new_target_opt)
        } else {
            let _ = self.output.lock(Duration::infinite()).map(|mut guard| {
                guard.targets[ch as usize] = target;
                guard.results[ch as usize] = None;
                guard.frequencys[ch as usize] = None;
                guard.values[ch as usize] = None;
            });
            (false, new_target_opt)
        }
    }

    fn process_f_signal_lost(&mut self, ch: FChannel, target: u32) -> (bool, Option<(u32, u32)>) {
        if let Ok(mut guard) = self.output.lock(Duration::infinite()) {
            guard.targets[ch as usize] = target;
            guard.results[ch as usize] = None;
            guard.frequencys[ch as usize] = None;
            guard.values[ch as usize] = None;
        }

        let config = super::channel_config(ch);
        let guard_ticks = super::guard_ticks(ch, &self.sysclk);
        (
            config.enabled,
            Some((crate::config::INITIAL_FREQMETER_TARGET, guard_ticks)),
        )
    }

    fn process_adc_result(
        &mut self,
        ch: AChannel,
        current_period_ticks: u32,
        adc: &mut ADC,
        controller: &mut dyn AController,
    ) -> (bool, Option<u32>) {
        let raw_adc_value = controller.read(adc);

        match ch {
            AChannel::TCPU => super::process_t_cpu(
                self.output.as_ref(),
                current_period_ticks,
                adc.to_degrees_centigrade(raw_adc_value),
                raw_adc_value,
                self.sysclk,
            ),
            AChannel::Vbat => super::process_vbat(
                self.output.as_ref(),
                current_period_ticks,
                adc.to_millivolts(raw_adc_value),
                raw_adc_value,
                self.sysclk,
            ),
        }
    }
}
