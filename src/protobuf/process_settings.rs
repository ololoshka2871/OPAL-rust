use alloc::{format, string::String};
use freertos_rust::{Duration, FreeRtosError};
use my_proc_macro::store_coeff;

use super::messages::{
    ru_sktbelpa_pressure_self_writer_PCoefficients,
    ru_sktbelpa_pressure_self_writer_SettingsResponse,
    ru_sktbelpa_pressure_self_writer_T5Coefficients,
    ru_sktbelpa_pressure_self_writer_WriteSettingsReq,
};

static MAX_MT: u32 = 5000;
static MIN_MT: u32 = 10;

static F_REF_BASE: u32 = 16000000;
static F_REF_DELTA: u32 = 500;

pub fn fill_settings(
    settings_resp: &mut ru_sktbelpa_pressure_self_writer_SettingsResponse,
) -> Result<(), ()> {
    crate::settings::settings_action(Duration::ms(1), |(ws, ts)| {
        settings_resp.Serial = ws.Serial;

        settings_resp.PMesureTime_ms = ws.PMesureTime_ms;
        settings_resp.TMesureTime_ms = ws.TMesureTime_ms;

        settings_resp.Fref = ws.Fref;

        settings_resp.PEnabled = ws.P_enabled;
        settings_resp.TEnabled = ws.T_enabled;

        settings_resp.PCoefficients = ru_sktbelpa_pressure_self_writer_PCoefficients {
            has_Fp0: true,
            Fp0: ws.P_Coefficients.Fp0,
            has_Ft0: true,
            Ft0: ws.P_Coefficients.Ft0,

            has_A0: true,
            A0: ws.P_Coefficients.A[0],
            has_A1: true,
            A1: ws.P_Coefficients.A[1],
            has_A2: true,
            A2: ws.P_Coefficients.A[2],
            has_A3: true,
            A3: ws.P_Coefficients.A[3],
            has_A4: true,
            A4: ws.P_Coefficients.A[4],
            has_A5: true,
            A5: ws.P_Coefficients.A[5],
            has_A6: true,
            A6: ws.P_Coefficients.A[6],
            has_A7: true,
            A7: ws.P_Coefficients.A[7],
            has_A8: true,
            A8: ws.P_Coefficients.A[8],
            has_A9: true,
            A9: ws.P_Coefficients.A[9],
            has_A10: true,
            A10: ws.P_Coefficients.A[10],
            has_A11: true,
            A11: ws.P_Coefficients.A[11],
            has_A12: true,
            A12: ws.P_Coefficients.A[12],
            has_A13: true,
            A13: ws.P_Coefficients.A[13],
            has_A14: true,
            A14: ws.P_Coefficients.A[14],
            has_A15: true,
            A15: ws.P_Coefficients.A[15],
        };

        settings_resp.TCoefficients = ru_sktbelpa_pressure_self_writer_T5Coefficients {
            has_T0: true,
            T0: ws.T_Coefficients.T0,
            has_F0: true,
            F0: ws.T_Coefficients.F0,

            has_C1: true,
            C1: ws.T_Coefficients.C[0],
            has_C2: true,
            C2: ws.T_Coefficients.C[1],
            has_C3: true,
            C3: ws.T_Coefficients.C[2],
            has_C4: true,
            C4: ws.T_Coefficients.C[3],
            has_C5: true,
            C5: ws.T_Coefficients.C[4],
        };
        Ok(())
    })
    .map_err(|_: crate::settings::SettingActionError<()>| ())
}

fn verify_parameters(
    ws: &ru_sktbelpa_pressure_self_writer_WriteSettingsReq,
) -> Result<(), crate::settings::SettingActionError<String>> {
    if ws.has_setPMesureTime_ms && (ws.setPMesureTime_ms > MAX_MT || ws.setPMesureTime_ms < MIN_MT)
    {
        return Err(crate::settings::SettingActionError::ActionError(format!(
            "Pressure measure time {} is out of range {} - {}",
            ws.setPMesureTime_ms, MIN_MT, MAX_MT
        )));
    }

    if ws.has_setTMesureTime_ms && (ws.setTMesureTime_ms > MAX_MT || ws.setTMesureTime_ms < MIN_MT)
    {
        return Err(crate::settings::SettingActionError::ActionError(format!(
            "Temperature measure time {} is out of range {} - {}",
            ws.setPMesureTime_ms, MIN_MT, MAX_MT
        )));
    }

    if ws.has_setFref
        && (ws.setFref > F_REF_BASE + F_REF_DELTA || ws.setFref < F_REF_BASE - F_REF_DELTA)
    {
        return Err(crate::settings::SettingActionError::ActionError(format!(
            "Reference frequency {} is too different from base {} +/- {}",
            ws.setFref, F_REF_BASE, F_REF_DELTA
        )));
    }

    Ok(())
}

pub fn update_settings(
    w: &ru_sktbelpa_pressure_self_writer_WriteSettingsReq,
) -> Result<(), crate::settings::SettingActionError<String>> {
    verify_parameters(w)?;

    let need_write = crate::settings::settings_action(Duration::ms(1), |(ws, ts)| {
        let mut need_write = false;

        // раскладывается в ->
        /*
        if w.has_setPMesureTime_ms {
            ws.PMesureTime_ms = w.setPMesureTime_ms;
            need_write = true;
        }*/
        store_coeff!(ws.PMesureTime_ms <= w; setPMesureTime_ms; need_write);
        
        if w.has_setTMesureTime_ms {
            ws.TMesureTime_ms = w.setTMesureTime_ms;
            need_write = true;
        }

        if w.has_setFref {
            ws.Fref = w.setFref;
            need_write = true;
        }

        if w.has_setSerial {
            ws.Serial = w.setSerial;
            need_write = true;
        }

        if w.has_setPEnabled {
            ws.P_enabled = w.setPEnabled;
            need_write = true;
        }

        if w.has_setTEnabled {
            ws.T_enabled = w.setTEnabled;
            need_write = true;
        }

        // TODO: заменить макросом эту порнуху
        if w.has_setPCoefficients {
            if w.setPCoefficients.has_Fp0 {
                ws.P_Coefficients.Fp0 = w.setPCoefficients.Fp0;
                need_write = true;
            }

            /*
            if ws.PCoefficients.has_Ft0 {
                ws.P_Coefficients.Ft0 = ws.PCoefficients.Ft0;
                need_write = true;
            }

            if ws.PCoefficients.has_A0 {
                ws.P_Coefficients.A[0] = ws.PCoefficients.A0;
                need_write = true;
            }

            if ws.PCoefficients.has_A1 {
                ws.P_Coefficients.A[1] = ws.PCoefficients.A1;
                need_write = true;
            }

            if ws.PCoefficients.has_A2 {
                ws.P_Coefficients.A[2] = ws.PCoefficients.A2;
                need_write = true;
            }

            if ws.PCoefficients.has_A3 {
                ws.P_Coefficients.A[3] = ws.PCoefficients.A3;
                need_write = true;
            }

            if ws.PCoefficients.has_A4 {
                ws.P_Coefficients.A[4] = ws.PCoefficients.A4;
                need_write = true;
            }

            if ws.PCoefficients.has_A5 {
                ws.P_Coefficients.A[5] = ws.PCoefficients.A5;
                need_write = true;
            }

            if ws.PCoefficients.has_A6 {
                ws.P_Coefficients.A[6] = ws.PCoefficients.A6;
                need_write = true;
            }

            if ws.PCoefficients.has_A7 {
                ws.P_Coefficients.A[7] = ws.PCoefficients.A7;
                need_write = true;
            }

            if ws.PCoefficients.has_A8 {
                ws.P_Coefficients.A[8] = ws.PCoefficients.A8;
                need_write = true;
            }

            if ws.PCoefficients.has_A9 {
                ws.P_Coefficients.A[9] = ws.PCoefficients.A9;
                need_write = true;
            }

            if ws.PCoefficients.has_A10 {
                ws.P_Coefficients.A[10] = ws.PCoefficients.A10;
                need_write = true;
            }

            if ws.PCoefficients.has_A11 {
                ws.P_Coefficients.A[11] = ws.PCoefficients.A11;
                need_write = true;
            }

            if ws.PCoefficients.has_A12 {
                ws.P_Coefficients.A[12] = ws.PCoefficients.A12;
                need_write = true;
            }

            if ws.PCoefficients.has_A13 {
                ws.P_Coefficients.A[13] = ws.PCoefficients.A13;
                need_write = true;
            }

            if ws.PCoefficients.has_A14 {
                ws.P_Coefficients.A[14] = ws.PCoefficients.A14;
                need_write = true;
            }

            if ws.PCoefficients.has_A15 {
                ws.P_Coefficients.A[15] = ws.PCoefficients.A15;
                need_write = true;
            }
            */
        }

        if w.has_setTCoefficients {
            if w.setTCoefficients.has_F0 {
                ws.T_Coefficients.F0 = w.setTCoefficients.F0;
                need_write = true;
            }
            /*
            if w.TCoefficients.has_T0 {
                ws.T_Coefficients.T0 = ws.TCoefficients.T0;
                need_write = true;
            }

            if ws.TCoefficients.has_C1 {
                ws.T_Coefficients.C[0] = ws.TCoefficients.C1;
                need_write = true;
            }

            if ws.TCoefficients.has_C2 {
                ws.T_Coefficients.C[1] = ws.TCoefficients.C2;
                need_write = true;
            }

            if ws.TCoefficients.has_C3 {
                ws.T_Coefficients.C[2] = ws.TCoefficients.C3;
                need_write = true;
            }

            if ws.TCoefficients.has_C4 {
                ws.T_Coefficients.C[3] = ws.TCoefficients.C4;
                need_write = true;
            }

            if ws.TCoefficients.has_C5 {
                ws.T_Coefficients.C[4] = ws.TCoefficients.C5;
                need_write = true;
            }
            */
        }

        Ok(need_write)
    })?;

    if need_write {
        start_writing_settings().map_err(|e| crate::settings::SettingActionError::AccessError(e))
    } else {
        Ok(())
    }
}

fn start_writing_settings() -> Result<(), FreeRtosError> {
    use freertos_rust::{Task, TaskPriority};
    defmt::warn!("Save settings rquested...");

    Task::new()
        .name("SS")
        .stack_size(384)
        .priority(TaskPriority(1))
        .start(move |_| {
            if let Err(e) = crate::settings::settings_save(Duration::infinite()) {
                defmt::error!("Failed to store settings: {}", defmt::Debug2Format(&e));
            }
        })
        .map(|_| ())?;

    Ok(())
}
