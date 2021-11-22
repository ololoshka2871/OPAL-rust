use super::messages::{
    ru_sktbelpa_pressure_self_writer_OutputReq, ru_sktbelpa_pressure_self_writer_OutputResponse,
};

pub fn fill_output(
    output: &mut ru_sktbelpa_pressure_self_writer_OutputResponse,
    get_output_values: &ru_sktbelpa_pressure_self_writer_OutputReq,
) -> Result<(), ()> {
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
        // TODO: values
        output.FP = 1.2e+2;
        output.FT = 1.3e+3;
    }

    if get_output_values.has_getRAW {
        output.has_P_result = true;
        output.has_T_result = true;
        output.has_ADC_TCPU = true;
        output.has_ADC_Vbat = true;
        // TODO: values
        output.P_result.Target = 1001;
        output.P_result.Result = 123456;
        output.T_result.Target = 999;
        output.T_result.Result = 123416;
        output.ADC_TCPU = 10358;
        output.ADC_Vbat = 18973;
    }

    Ok(())
}
