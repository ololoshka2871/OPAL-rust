use lazy_static::lazy_static;

use alloc::sync::Arc;
use freertos_rust::{Duration, FreeRtosError, Mutex};

use crate::{threads::sensor_processor::FChannel, workmodes::output_storage::OutputStorage};

use super::messages::{
    ru_sktbelpa_pressure_self_writer_OutputReq, ru_sktbelpa_pressure_self_writer_OutputResponse,
};

lazy_static! {
    static ref OUT_STORAGE_LOCK_WAIT: Duration = Duration::ms(5);
}

pub fn fill_output(
    output: &mut ru_sktbelpa_pressure_self_writer_OutputResponse,
    get_output_values: &ru_sktbelpa_pressure_self_writer_OutputReq,
    output_storage: &Arc<Mutex<OutputStorage>>,
) -> Result<(), FreeRtosError> {
    let mut err = None;

    if get_output_values.has_getMainValues {
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
            }
            Err(e) => {
                output.pressure = f32::NAN;
                output.temperature = f32::NAN;
                err = Some(e);
            }
        }

        // TODO: values
        output.TCPU = 1.3e-3;
        output.Vbat = 1.4e-4;
    }

    if get_output_values.has_getF {
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

    if get_output_values.has_getRAW {
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
            }
            Err(e) => {
                output.P_result.Target = 0;
                output.P_result.Result = 0;
                output.T_result.Target = 0;
                output.T_result.Result = 0;
                err = Some(e);
            }
        }

        // TODO: values
        output.ADC_TCPU = 10358;
        output.ADC_Vbat = 18973;
    }

    if err.is_some() {
        Err(err.unwrap())
    } else {
        Ok(())
    }
}
