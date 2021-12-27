use lazy_static::lazy_static;

use alloc::sync::Arc;
use freertos_rust::{Duration, FreeRtosError, Mutex};

use crate::{threads::sensor_processor::FChannel, workmodes::output_storage::OutputStorage};

lazy_static! {
    static ref OUT_STORAGE_LOCK_WAIT: Duration = Duration::ms(5);
}

pub fn fill_output(
    output: &mut super::messages::OutputResponse,
    get_output_values: &super::messages::OutputReq,
    output_storage: &Arc<Mutex<OutputStorage>>,
) -> Result<(), FreeRtosError> {
    let mut err = None;

    if get_output_values.get_main_values.is_some() {
        match output_storage.lock(*OUT_STORAGE_LOCK_WAIT) {
            Ok(guard) => {
                output.pressure =
                    Some(guard.values[FChannel::Pressure as usize].unwrap_or(f64::NAN) as f32);
                output.temperature =
                    Some(guard.values[FChannel::Temperature as usize].unwrap_or(f64::NAN) as f32);
                output.tcpu = Some(guard.t_cpu);
                output.vbat = Some(guard.vbat as f32);
            }
            Err(e) => {
                output.pressure = Some(f32::NAN);
                output.temperature = Some(f32::NAN);
                output.tcpu = Some(f32::NAN);
                output.vbat = Some(f32::NAN);
                err = Some(e);
            }
        }
    }

    if get_output_values.get_f.is_some() {
        match output_storage.lock(*OUT_STORAGE_LOCK_WAIT) {
            Ok(guard) => {
                output.fp =
                    Some(guard.frequencys[FChannel::Pressure as usize].unwrap_or(f64::NAN) as f32);
                output.ft = Some(
                    guard.frequencys[FChannel::Temperature as usize].unwrap_or(f64::NAN) as f32,
                );
            }
            Err(e) => {
                output.fp = Some(f32::NAN);
                output.ft = Some(f32::NAN);
                err = Some(e);
            }
        }
    }

    if get_output_values.get_raw.is_some() {
        match output_storage.lock(*OUT_STORAGE_LOCK_WAIT) {
            Ok(guard) => {
                output.p_result = Some(super::messages::FreqmeterResult {
                    target: guard.targets[FChannel::Pressure as usize],
                    result: guard.results[FChannel::Pressure as usize].unwrap_or_default(),
                });
                output.t_result = Some(super::messages::FreqmeterResult {
                    target: guard.targets[FChannel::Temperature as usize],
                    result: guard.results[FChannel::Temperature as usize].unwrap_or_default(),
                });

                output.adc_tcpu = Some(guard.t_cpu_adc as u32);
                output.adc_vbat = Some(guard.vbat_adc as u32);
            }
            Err(e) => {
                output.p_result = Some(super::messages::FreqmeterResult::default());
                output.t_result = Some(super::messages::FreqmeterResult::default());

                output.adc_tcpu = Some(u32::default());
                output.adc_vbat = Some(u32::default());
                err = Some(e);
            }
        }
    }

    if let Some(err) = err {
        Err(err)
    } else {
        Ok(())
    }
}
