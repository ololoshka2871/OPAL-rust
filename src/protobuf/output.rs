use lazy_static::lazy_static;

use alloc::sync::Arc;
use freertos_rust::{Duration, FreeRtosError, Mutex};

use crate::{threads::sensor_processor::FChannel, workmodes::output_storage::OutputStorage};

use super::messages::ru_sktbelpa_pressure_self_writer_OutputResponse;

lazy_static! {
    static ref OUT_STORAGE_LOCK_WAIT: Duration = Duration::ms(5);
}

pub fn fill_output(
    output: &mut ru_sktbelpa_pressure_self_writer_OutputResponse,
    get_output_values: &super::messages::OutputReq,
    output_storage: &Arc<Mutex<OutputStorage>>,
) -> Result<(), FreeRtosError> {
    let mut err = None;

    if get_output_values.get_main_values.is_some() {
        output.has_pressure = true;
        output.has_temperature = true;
        output.has_TCPU = true;
        output.has_Vbat = true;

        match output_storage.lock(*OUT_STORAGE_LOCK_WAIT) {
            Ok(guard) => {
                output.pressure =
                    guard.values[FChannel::Pressure as usize].unwrap_or(f64::NAN) as f32;
                output.temperature =
                    guard.values[FChannel::Temperature as usize].unwrap_or(f64::NAN) as f32;
                output.TCPU = guard.t_cpu;
                output.Vbat = guard.vbat_mv as f32;
            }
            Err(e) => {
                output.pressure = f32::NAN;
                output.temperature = f32::NAN;
                output.TCPU = f32::NAN;
                output.Vbat = f32::NAN;
                err = Some(e);
            }
        }
    }

    if get_output_values.get_f.is_some() {
        output.has_FP = true;
        output.has_FT = true;

        match output_storage.lock(*OUT_STORAGE_LOCK_WAIT) {
            Ok(guard) => {
                output.FP =
                    guard.frequencys[FChannel::Pressure as usize].unwrap_or(f64::NAN) as f32;
                output.FT =
                    guard.frequencys[FChannel::Temperature as usize].unwrap_or(f64::NAN) as f32;
            }
            Err(e) => {
                output.FP = f32::NAN;
                output.FT = f32::NAN;
                err = Some(e);
            }
        }
    }

    if get_output_values.get_raw.is_some() {
        output.has_P_result = true;
        output.has_T_result = true;
        output.has_ADC_TCPU = true;
        output.has_ADC_Vbat = true;

        match output_storage.lock(*OUT_STORAGE_LOCK_WAIT) {
            Ok(guard) => {
                output.P_result.Target = guard.targets[FChannel::Pressure as usize];
                output.P_result.Result =
                    guard.results[FChannel::Pressure as usize].unwrap_or_default();
                output.T_result.Target = guard.targets[FChannel::Temperature as usize];
                output.T_result.Result =
                    guard.results[FChannel::Temperature as usize].unwrap_or_default();

                output.ADC_TCPU = guard.t_cpu_adc as u32;
                output.ADC_Vbat = guard.vbat_mv_adc as u32;
            }
            Err(e) => {
                output.P_result.Target = 0;
                output.P_result.Result = 0;
                output.T_result.Target = 0;
                output.T_result.Result = 0;

                output.ADC_TCPU = 0;
                output.ADC_Vbat = 0;
                err = Some(e);
            }
        }
    }

    if err.is_some() {
        Err(err.unwrap())
    } else {
        Ok(())
    }
}
