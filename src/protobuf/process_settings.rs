use alloc::{
    format,
    string::{String, ToString},
};
use freertos_rust::{Duration, FreeRtosError};
use my_proc_macro::store_coeff;

use crate::{protobuf::PASSWORD_SIZE, settings::SettingActionError};

use super::messages::{
    ru_sktbelpa_pressure_self_writer_CalibrationDate,
    ru_sktbelpa_pressure_self_writer_PCoefficients,
    ru_sktbelpa_pressure_self_writer_SettingsResponse,
    ru_sktbelpa_pressure_self_writer_T5Coefficients, ru_sktbelpa_pressure_self_writer_WorkRange,
    ru_sktbelpa_pressure_self_writer_WriteConfig,
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
        settings_resp.TCPUEnabled = ws.TCPUEnabled;
        settings_resp.VBatEnable = ws.VBatEnable;

        settings_resp.PCoefficients =
            ru_sktbelpa_pressure_self_writer_PCoefficients::from(&ws.P_Coefficients);
        settings_resp.TCoefficients =
            ru_sktbelpa_pressure_self_writer_T5Coefficients::from(&ws.T_Coefficients);

        settings_resp.PWorkRange = ru_sktbelpa_pressure_self_writer_WorkRange::from(&ws.PWorkRange);
        settings_resp.TWorkRange = ru_sktbelpa_pressure_self_writer_WorkRange::from(&ws.TWorkRange);
        settings_resp.TCPUWorkRange =
            ru_sktbelpa_pressure_self_writer_WorkRange::from(&ws.TCPUWorkRange);
        settings_resp.BatWorkRange =
            ru_sktbelpa_pressure_self_writer_WorkRange::from(&ws.VbatWorkRange);

        settings_resp.CalibrationDate =
            ru_sktbelpa_pressure_self_writer_CalibrationDate::from(&ws.calibration_date);

        settings_resp.PZeroCorrection = ws.PZeroCorrection;
        settings_resp.TZeroCorrection = ws.TZeroCorrection;

        settings_resp.writeConfig =
            ru_sktbelpa_pressure_self_writer_WriteConfig::from(&ws.writeConfig);

        settings_resp.startDelay = ws.startDelay;

        settings_resp.password.copy_from_slice(&ts.current_password);

        Ok(())
    })
    .map_err(|_: SettingActionError<()>| ())
}

fn verify_parameters(
    ws: &ru_sktbelpa_pressure_self_writer_WriteSettingsReq,
) -> Result<(), SettingActionError<String>> {
    let password_invalid = crate::settings::settings_action(Duration::ms(1), |(ws, ts)| {
        Ok(ws.password != ts.current_password)
    })?;

    let deny_if_password_invalid = || {
        if password_invalid {
            Err(SettingActionError::ActionError(
                "Rejected, invalid password".to_string(),
            ))
        } else {
            Ok(())
        }
    };

    if ws.has_setSerial {
        deny_if_password_invalid()?;
    }

    if ws.has_setPMesureTime_ms && (ws.setPMesureTime_ms > MAX_MT || ws.setPMesureTime_ms < MIN_MT)
    {
        return Err(SettingActionError::ActionError(format!(
            "Pressure measure time {} is out of range {} - {}",
            ws.setPMesureTime_ms, MIN_MT, MAX_MT
        )));
    }

    if ws.has_setTMesureTime_ms && (ws.setTMesureTime_ms > MAX_MT || ws.setTMesureTime_ms < MIN_MT)
    {
        return Err(SettingActionError::ActionError(format!(
            "Temperature measure time {} is out of range {} - {}",
            ws.setPMesureTime_ms, MIN_MT, MAX_MT
        )));
    }

    if ws.has_setFref
        && (ws.setFref > F_REF_BASE + F_REF_DELTA || ws.setFref < F_REF_BASE - F_REF_DELTA)
    {
        return Err(SettingActionError::ActionError(format!(
            "Reference frequency {} is too different from base {} +/- {}",
            ws.setFref, F_REF_BASE, F_REF_DELTA
        )));
    }

    if ws.has_setPWorkRange {
        deny_if_password_invalid()?;
        ws.setPWorkRange
            .validate()
            .map_err(|e| SettingActionError::ActionError(format!("PWorkRange invalid: {:?}", e)))?;
    }

    if ws.has_setTWorkRange {
        deny_if_password_invalid()?;
        ws.setTWorkRange
            .validate()
            .map_err(|e| SettingActionError::ActionError(format!("TWorkRange invalid: {:?}", e)))?;
    }

    if ws.has_setTCPUWorkRange {
        deny_if_password_invalid()?;
        ws.setTCPUWorkRange.validate().map_err(|e| {
            SettingActionError::ActionError(format!("TCPUWorkRange invalid: {:?}", e))
        })?;
    }

    if ws.has_setBatWorkRange {
        deny_if_password_invalid()?;
        ws.setBatWorkRange.validate().map_err(|e| {
            SettingActionError::ActionError(format!("BatWorkRange invalid: {:?}", e))
        })?;
    }

    if ws.has_setCalibrationDate {
        ws.setCalibrationDate.validate().map_err(|e| {
            SettingActionError::ActionError(format!("Calibration date field {:?} invalid", e))
        })?;
    }

    if ws.has_setWriteConfig {
        if ws.setWriteConfig.BaseInterval_ms < MIN_MT {
            return Err(SettingActionError::ActionError(format!(
                "Write base period {} too small, min= {}",
                ws.setWriteConfig.BaseInterval_ms, MIN_MT
            )));
        }
    }

    Ok(())
}

pub fn update_settings(
    w: &ru_sktbelpa_pressure_self_writer_WriteSettingsReq,
) -> Result<(), SettingActionError<String>> {
    verify_parameters(w)?;

    let need_write = crate::settings::settings_action(Duration::ms(1), |(ws, ts)| {
        let mut need_write = false;

        store_coeff!(ws.Serial <= w; setSerial; need_write);

        // раскладывается в ->
        /*
        if w.has_setPMesureTime_ms {
            ws.PMesureTime_ms = w.setPMesureTime_ms;
            need_write = true;
        }*/
        store_coeff!(ws.PMesureTime_ms <= w; setPMesureTime_ms; need_write);
        store_coeff!(ws.TMesureTime_ms <= w; setTMesureTime_ms; need_write);

        store_coeff!(ws.Fref <= w; setFref; need_write);

        store_coeff!(ws.P_enabled <= w; setPEnabled; need_write);
        store_coeff!(ws.T_enabled <= w; setTEnabled; need_write);
        store_coeff!(ws.TCPUEnabled <= w; setTCPUEnabled; need_write);
        store_coeff!(ws.VBatEnable <= w; setVBatEnable; need_write);

        if w.has_setPCoefficients {
            store_coeff!(ws.P_Coefficients.Fp0 <= w.setPCoefficients; Fp0; need_write);
            store_coeff!(ws.P_Coefficients.Ft0 <= w.setPCoefficients; Ft0; need_write);
            store_coeff!(ws.P_Coefficients.A[0] <= w.setPCoefficients; A0; need_write);
            store_coeff!(ws.P_Coefficients.A[1] <= w.setPCoefficients; A1; need_write);
            store_coeff!(ws.P_Coefficients.A[2] <= w.setPCoefficients; A2; need_write);
            store_coeff!(ws.P_Coefficients.A[3] <= w.setPCoefficients; A3; need_write);
            store_coeff!(ws.P_Coefficients.A[4] <= w.setPCoefficients; A4; need_write);
            store_coeff!(ws.P_Coefficients.A[5] <= w.setPCoefficients; A5; need_write);
            store_coeff!(ws.P_Coefficients.A[6] <= w.setPCoefficients; A6; need_write);
            store_coeff!(ws.P_Coefficients.A[7] <= w.setPCoefficients; A7; need_write);
            store_coeff!(ws.P_Coefficients.A[8] <= w.setPCoefficients; A8; need_write);
            store_coeff!(ws.P_Coefficients.A[9] <= w.setPCoefficients; A9; need_write);
            store_coeff!(ws.P_Coefficients.A[10] <= w.setPCoefficients; A10; need_write);
            store_coeff!(ws.P_Coefficients.A[11] <= w.setPCoefficients; A11; need_write);
            store_coeff!(ws.P_Coefficients.A[12] <= w.setPCoefficients; A12; need_write);
            store_coeff!(ws.P_Coefficients.A[13] <= w.setPCoefficients; A13; need_write);
            store_coeff!(ws.P_Coefficients.A[14] <= w.setPCoefficients; A14; need_write);
            store_coeff!(ws.P_Coefficients.A[15] <= w.setPCoefficients; A15; need_write);
        }

        if w.has_setTCoefficients {
            store_coeff!(ws.T_Coefficients.F0 <= w.setTCoefficients; F0; need_write);
            store_coeff!(ws.T_Coefficients.C[0] <= w.setTCoefficients; C1; need_write);
            store_coeff!(ws.T_Coefficients.C[1] <= w.setTCoefficients; C2; need_write);
            store_coeff!(ws.T_Coefficients.C[2] <= w.setTCoefficients; C3; need_write);
            store_coeff!(ws.T_Coefficients.C[3] <= w.setTCoefficients; C4; need_write);
            store_coeff!(ws.T_Coefficients.C[4] <= w.setTCoefficients; C5; need_write);
            store_coeff!(ws.T_Coefficients.T0 <= w.setTCoefficients; T0; need_write);
        }

        if w.has_setPWorkRange {
            store_coeff!(ws.PWorkRange.minimum <= w.setPWorkRange; minimum; need_write);
            store_coeff!(ws.PWorkRange.maximum <= w.setPWorkRange; maximum; need_write);
            store_coeff!(ws.PWorkRange.absolute_maximum <= w.setPWorkRange; absolute_maximum; need_write);
        }
        if w.has_setTWorkRange {
            store_coeff!(ws.TWorkRange.minimum <= w.setTWorkRange; minimum; need_write);
            store_coeff!(ws.TWorkRange.maximum <= w.setTWorkRange; maximum; need_write);
            store_coeff!(ws.TWorkRange.absolute_maximum <= w.setTWorkRange; absolute_maximum; need_write);
        }
        if w.has_setTCPUWorkRange {
            store_coeff!(ws.TCPUWorkRange.minimum <= w.setTCPUWorkRange; minimum; need_write);
            store_coeff!(ws.TCPUWorkRange.maximum <= w.setTCPUWorkRange; maximum; need_write);
            store_coeff!(ws.TCPUWorkRange.absolute_maximum <= w.setTCPUWorkRange; absolute_maximum; need_write);
        }
        if w.has_setBatWorkRange {
            store_coeff!(ws.VbatWorkRange.minimum <= w.setBatWorkRange; minimum; need_write);
            store_coeff!(ws.VbatWorkRange.maximum <= w.setBatWorkRange; maximum; need_write);
            store_coeff!(ws.VbatWorkRange.absolute_maximum <= w.setBatWorkRange; absolute_maximum; need_write);
        }

        if w.has_setCalibrationDate {
            store_coeff!(ws.calibration_date.Day <= w.setCalibrationDate; Day; need_write);
            store_coeff!(ws.calibration_date.Month <= w.setCalibrationDate; Month; need_write);
            store_coeff!(ws.calibration_date.Year <= w.setCalibrationDate; Year; need_write);
        }

        store_coeff!(ws.PZeroCorrection <= w; setPZeroCorrection; need_write);
        store_coeff!(ws.TZeroCorrection <= w; setTZeroCorrection; need_write);

        if w.has_setWriteConfig {
            store_coeff!(ws.writeConfig.BaseInterval_ms <= w.setWriteConfig; BaseInterval_ms; need_write);
            store_coeff!(ws.writeConfig.PWriteDevider <= w.setWriteConfig; PWriteDevider; need_write);
            store_coeff!(ws.writeConfig.TWriteDevider <= w.setWriteConfig; TWriteDevider; need_write);
        }

        store_coeff!(ws.startDelay <= w; setStartDelay; need_write);

        if w.has_setPassword {
            ts.current_password
                .copy_from_slice(&w.setPassword[..PASSWORD_SIZE]);
        }

        Ok(need_write)
    })?;

    if need_write {
        start_writing_settings().map_err(|e| SettingActionError::AccessError(e))
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
