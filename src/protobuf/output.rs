use lazy_static::lazy_static;

use alloc::sync::Arc;
use freertos_rust::{Duration, FreeRtosError, Mutex};

use crate::workmodes::output_storage::OutputStorage;

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
        // TODO: values
        output.pressure = 1.1e-1;
        output.temperature = 1.2e-2;
        output.TCPU = 1.3e-3;
        output.Vbat = 1.4e-4;
    }

    if get_output_values.has_getF {
        output.has_FP = true;
        output.has_FT = true;

        match output_storage.lock(*OUT_STORAGE_LOCK_WAIT) {
            Ok(guard) => {
                output.FP = guard.frequencys[0] as f32;
                output.FT = guard.frequencys[1] as f32;
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
                output.P_result.Target = guard.targets[0];
                output.P_result.Result = guard.results[0].unwrap_or_default();
                output.T_result.Target = guard.targets[1];
                output.T_result.Result = guard.results[1].unwrap_or_default();
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
