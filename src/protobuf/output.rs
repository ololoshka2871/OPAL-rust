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
    }

    if get_output_values.has_getF {
        output.has_FP = true;
        output.has_FT = true;
        // TODO: values
    }

    if get_output_values.has_getRAW {
        output.has_P_result = true;
        output.has_T_result = true;
        output.has_ADC_TCPU = true;
        output.has_Vbat = true;
        // TODO: values
    }

    Ok(())
}
