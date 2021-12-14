use alloc::sync::Arc;
use freertos_rust::{Duration, Mutex};
use stm32l4xx_hal::time::Hertz;

use crate::{
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
        if let Ok(config) = super::channel_config(ch) {
            let mut new_target_opt = None;
            if config.enabled {
                if let Ok(mut guard) = self.output.lock(Duration::infinite()) {
                    guard.targets[ch as usize] = target;
                    guard.results[ch as usize] = Some(result);
                }

                if let Ok(f) = super::calc_freq(self.fref_multiplier, target, result) {
                    if let Ok(mut guard) = self.output.lock(Duration::infinite()) {
                        guard.frequencys[ch as usize] = Some(f);
                    }

                    match ch {
                        FChannel::Pressure => super::calc_pressure(f, self.output.as_ref()),
                        FChannel::Temperature => super::calc_temperature(f, self.output.as_ref()),
                    }

                    if let Ok((new_target, guard_ticks)) =
                        super::calc_new_target(ch, f, &self.sysclk)
                    {
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
                    }
                }
                (true, new_target_opt)
            } else {
                self.output
                    .lock(Duration::infinite())
                    .map(|mut guard| {
                        guard.targets[ch as usize] = target;
                        guard.results[ch as usize] = None;
                        guard.frequencys[ch as usize] = None;
                        guard.values[ch as usize] = None;
                    })
                    .unwrap();
                (false, new_target_opt)
            }
        } else {
            defmt::error!("Failed to read channel config, abort processing.");
            (true, None)
        }
    }

    fn process_f_signal_lost(&mut self, ch: FChannel, target: u32) -> (bool, Option<(u32, u32)>) {
        if let Ok(mut guard) = self.output.lock(Duration::infinite()) {
            guard.targets[ch as usize] = target;
            guard.results[ch as usize] = None;
            guard.frequencys[ch as usize] = None;
            guard.values[ch as usize] = None;
        }

        if let Ok(config) = super::channel_config(ch) {
            if let Ok(guard_ticks) = super::guard_ticks(ch, &self.sysclk) {
                return (
                    config.enabled,
                    Some((crate::config::INITIAL_FREQMETER_TARGET, guard_ticks)),
                );
            }
        }

        (false, None)
    }

    fn process_adc_result(&mut self, _ch: AChannel, _result: u32) -> bool {
        todo!()
    }
}