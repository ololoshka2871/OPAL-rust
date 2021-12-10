use alloc::sync::Arc;
use freertos_rust::{Duration, Mutex};

use crate::{
    threads::sensor_processor::{AChannel, FChannel},
    workmodes::output_storage::OutputStorage,
};

use super::RawValueProcessor;

pub struct HighPerformanceProcessor {
    output: Arc<Mutex<OutputStorage>>,
    fref_multiplier: f64,
}

impl HighPerformanceProcessor {
    pub fn new(output: Arc<Mutex<OutputStorage>>, fref_multiplier: f64) -> Self {
        Self {
            output,
            fref_multiplier,
        }
    }
}

impl RawValueProcessor for HighPerformanceProcessor {
    fn process_f_result(&mut self, ch: FChannel, target: u32, result: u32) -> (bool, Option<u32>) {
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

                    if let Ok(new_target) = super::calc_new_target(ch, f) {
                        if super::abs_difference(new_target, target)
                            > crate::config::MINIMUM_ADAPTATION_INTERVAL
                        {
                            defmt::warn!("Adaptation ch. {}, new target {}", ch, new_target);
                            new_target_opt = Some(new_target);
                        }
                    }
                }

                (true, new_target_opt)
            } else {
                if let Ok(mut guard) = self.output.lock(Duration::infinite()) {
                    guard.targets[ch as usize] = target;
                    guard.results[ch as usize] = None;
                    guard.frequencys[ch as usize] = None;
                    guard.values[ch as usize] = None;
                }
                (false, new_target_opt)
            }
        } else {
            defmt::error!("Failed to read channel config, abort processing.");
            (true, None)
        }
    }

    fn process_adc_result(&mut self, _ch: AChannel, _result: u32) -> bool {
        todo!()
    }
}
