use core::usize;

use alloc::{
    format,
    string::{String, ToString},
};

use freertos_rust::Duration;
use my_proc_macro::store_coeff;

use crate::{
    config::XTAL_FREQ,
    protobuf::PASSWORD_SIZE,
    settings::{SettingActionError, MAX_MT, MIN_MT},
};

const F_REF_DELTA: u32 = 500;

fn strlenn(str: &[u8], max: usize) -> usize {
    let max_scan = core::cmp::min(str.len(), max);
    for i in 0..max_scan {
        if str[i] == b'\0' {
            return i;
        }
    }
    max
}

pub fn fill_settings(settings_resp: &mut super::messages::SettingsResponse) -> Result<(), ()> {
    crate::settings::settings_action(Duration::ms(1), |(ws, ts)| {
        settings_resp.serial = ws.Serial;

        settings_resp.p_mesure_time_ms = ws.PMesureTime_ms;
        settings_resp.t_mesure_time_ms = ws.TMesureTime_ms;

        settings_resp.fref = ws.Fref;

        settings_resp.p_enabled = ws.P_enabled;
        settings_resp.t_enabled = ws.T_enabled;
        settings_resp.tcpu_enabled = ws.TCPUEnabled;
        settings_resp.v_bat_enable = ws.VBatEnabled;

        settings_resp.p_coefficients = super::messages::PCoefficients::from(&ws.P_Coefficients);
        settings_resp.t_coefficients = super::messages::T5Coefficients::from(&ws.T_Coefficients);

        settings_resp.p_work_range = super::messages::WorkRange::from(&ws.PWorkRange);
        settings_resp.t_work_range = super::messages::WorkRange::from(&ws.TWorkRange);
        settings_resp.tcpu_work_range = super::messages::WorkRange::from(&ws.TCPUWorkRange);
        settings_resp.bat_work_range = super::messages::WorkRange::from(&ws.VbatWorkRange);

        settings_resp.calibration_date =
            super::messages::CalibrationDate::from(&ws.calibration_date);

        settings_resp.p_zero_correction = ws.PZeroCorrection;
        settings_resp.t_zero_correction = ws.TZeroCorrection;

        settings_resp.write_config = super::messages::WriteConfig::from(&ws.writeConfig);

        settings_resp.start_delay = ws.startDelay;

        settings_resp.pressure_meassure_units = ws.pressureMeassureUnits as i32;

        settings_resp.password = String::from_utf8_lossy(
            &ts.current_password[..strlenn(&ts.current_password, PASSWORD_SIZE)],
        )
        .to_string();

        Ok(())
    })
    .map_err(|_: SettingActionError<()>| ())
}

fn verify_parameters(
    ws: &super::messages::WriteSettingsReq,
) -> Result<(), SettingActionError<String>> {
    let password_invalid = crate::settings::settings_action(Duration::ms(1), |(ws, ts)| {
        Ok(ws.password != ts.current_password)
    })?;

    let deny_if_password_invalid = |parameter: &str| {
        if password_invalid {
            Err(SettingActionError::ActionError(format!(
                "Change {}, invalid password",
                parameter
            )))
        } else {
            Ok(())
        }
    };

    if ws.set_serial.is_some() {
        deny_if_password_invalid("Serial")?;
    }

    if let Some(set_p_mesure_time_ms) = ws.set_p_mesure_time_ms {
        if set_p_mesure_time_ms > MAX_MT || set_p_mesure_time_ms < MIN_MT {
            return Err(SettingActionError::ActionError(format!(
                "Pressure measure time {} is out of range {} - {}",
                set_p_mesure_time_ms, MIN_MT, MAX_MT
            )));
        }
    }

    if let Some(set_t_mesure_time_ms) = ws.set_t_mesure_time_ms {
        if set_t_mesure_time_ms > MAX_MT || set_t_mesure_time_ms < MIN_MT {
            return Err(SettingActionError::ActionError(format!(
                "Temperature measure time {} is out of range {} - {}",
                set_t_mesure_time_ms, MIN_MT, MAX_MT
            )));
        }
    }

    if let Some(set_fref) = ws.set_fref {
        deny_if_password_invalid("Fref")?;
        if set_fref > XTAL_FREQ + F_REF_DELTA || set_fref < XTAL_FREQ - F_REF_DELTA {
            return Err(SettingActionError::ActionError(format!(
                "Reference frequency {} is too different from base {} +/- {}",
                set_fref, XTAL_FREQ, F_REF_DELTA
            )));
        }
    }

    if let Some(set_p_coefficients) = &ws.set_p_coefficients {
        if set_p_coefficients.a0.is_some()
            || set_p_coefficients.a1.is_some()
            || set_p_coefficients.a2.is_some()
            || set_p_coefficients.a3.is_some()
            || set_p_coefficients.a4.is_some()
            || set_p_coefficients.a5.is_some()
            || set_p_coefficients.a6.is_some()
            || set_p_coefficients.a7.is_some()
            || set_p_coefficients.a8.is_some()
            || set_p_coefficients.a9.is_some()
            || set_p_coefficients.a10.is_some()
            || set_p_coefficients.a11.is_some()
            || set_p_coefficients.a12.is_some()
            || set_p_coefficients.a13.is_some()
            || set_p_coefficients.a14.is_some()
            || set_p_coefficients.a15.is_some()
            || set_p_coefficients.ft0.is_some()
            || set_p_coefficients.fp0.is_some()
        {
            deny_if_password_invalid("PCoefficients")?;
        }
    }

    if let Some(set_t_coefficients) = &ws.set_t_coefficients {
        if set_t_coefficients.t0.is_some()
            || set_t_coefficients.c1.is_some()
            || set_t_coefficients.c2.is_some()
            || set_t_coefficients.c3.is_some()
            || set_t_coefficients.c4.is_some()
            || set_t_coefficients.c5.is_some()
            || set_t_coefficients.f0.is_some()
        {
            deny_if_password_invalid("TCoefficients")?;
        }
    }

    if let Some(set_p_work_range) = &ws.set_p_work_range {
        if set_p_work_range.minimum.is_some()
            || set_p_work_range.maximum.is_some()
            || set_p_work_range.absolute_maximum.is_some()
        {
            deny_if_password_invalid("PWorkRange")?;

            set_p_work_range.validate().map_err(|e| {
                SettingActionError::ActionError(format!("PWorkRange invalid: {:?}", e))
            })?;
        }
    }

    if let Some(set_t_work_range) = &ws.set_t_work_range {
        if set_t_work_range.minimum.is_some()
            || set_t_work_range.maximum.is_some()
            || set_t_work_range.absolute_maximum.is_some()
        {
            deny_if_password_invalid("TWorkRange")?;

            set_t_work_range.validate().map_err(|e| {
                SettingActionError::ActionError(format!("TWorkRange invalid: {:?}", e))
            })?;
        }
    }

    if let Some(set_tcpu_work_range) = &ws.set_tcpu_work_range {
        if set_tcpu_work_range.minimum.is_some()
            || set_tcpu_work_range.maximum.is_some()
            || set_tcpu_work_range.absolute_maximum.is_some()
        {
            deny_if_password_invalid("TWorkRange")?;

            set_tcpu_work_range.validate().map_err(|e| {
                SettingActionError::ActionError(format!("TCPUWorkRange invalid: {:?}", e))
            })?;
        }
    }

    if let Some(set_bat_work_range) = &ws.set_bat_work_range {
        if set_bat_work_range.minimum.is_some()
            || set_bat_work_range.maximum.is_some()
            || set_bat_work_range.absolute_maximum.is_some()
        {
            deny_if_password_invalid("TWorkRange")?;

            set_bat_work_range.validate().map_err(|e| {
                SettingActionError::ActionError(format!("BatWorkRange invalid: {:?}", e))
            })?;
        }
    }

    if let Some(set_calibration_date) = &ws.set_calibration_date {
        set_calibration_date.validate().map_err(|e| {
            SettingActionError::ActionError(format!("Calibration date field {:?} invalid", e))
        })?;
    }

    if let Some(set_write_config) = &ws.set_write_config {
        if let Some(base_interval_ms) = set_write_config.base_interval_ms {
            if base_interval_ms < MIN_MT {
                return Err(SettingActionError::ActionError(format!(
                    "Write base period {} too small, min= {}",
                    base_interval_ms, MIN_MT
                )));
            }
        }
        if let Some(p_devider) = set_write_config.p_write_devider {
            if p_devider == 0 {
                return Err(SettingActionError::ActionError(
                    "P write devider == 0".to_string(),
                ));
            }
        }
        if let Some(t_devider) = set_write_config.t_write_devider {
            if t_devider == 0 {
                return Err(SettingActionError::ActionError(
                    "T write devider == 0".to_string(),
                ));
            }
        }
    }

    if let Some(set_pressure_meassure_units) = ws.set_pressure_meassure_units {
        if let Some(crate::settings::app_settings::PressureMeassureUnits::INVALID_ZERO) | None =
            num::FromPrimitive::from_i32(set_pressure_meassure_units)
        {
            return Err(SettingActionError::ActionError(format!(
                "Value {} is not a valid pressure measure unit code.",
                set_pressure_meassure_units
            )));
        }
    }

    Ok(())
}

pub fn update_settings(
    w: &super::messages::WriteSettingsReq,
) -> Result<bool, SettingActionError<String>> {
    verify_parameters(w)?;

    crate::settings::settings_action(Duration::ms(1), |(ws, ts)| {
        let mut need_write = false;

        // раскладывается в ->
        /*
        w.set_serial.map(|v| {
            ws.Serial = v;
            need_write = true;
        });
        */
        store_coeff!(ws.Serial <= w; set_serial; need_write);

        store_coeff!(ws.PMesureTime_ms <= w; set_p_mesure_time_ms; need_write);
        store_coeff!(ws.TMesureTime_ms <= w; set_t_mesure_time_ms; need_write);

        store_coeff!(ws.Fref <= w; set_fref; need_write);

        store_coeff!(ws.P_enabled <= w; set_p_enabled; need_write);
        store_coeff!(ws.T_enabled <= w; set_t_enabled; need_write);
        store_coeff!(ws.TCPUEnabled <= w; set_tcpu_enabled; need_write);
        store_coeff!(ws.VBatEnabled <= w; set_v_bat_enable; need_write);

        if let Some(set_p_coefficients) = &w.set_p_coefficients {
            store_coeff!(ws.P_Coefficients.Fp0 <= set_p_coefficients; fp0; need_write);
            store_coeff!(ws.P_Coefficients.Ft0 <= set_p_coefficients; ft0; need_write);
            store_coeff!(ws.P_Coefficients.A[0] <= set_p_coefficients; a0; need_write);
            store_coeff!(ws.P_Coefficients.A[1] <= set_p_coefficients; a1; need_write);
            store_coeff!(ws.P_Coefficients.A[2] <= set_p_coefficients; a2; need_write);
            store_coeff!(ws.P_Coefficients.A[3] <= set_p_coefficients; a3; need_write);
            store_coeff!(ws.P_Coefficients.A[4] <= set_p_coefficients; a4; need_write);
            store_coeff!(ws.P_Coefficients.A[5] <= set_p_coefficients; a5; need_write);
            store_coeff!(ws.P_Coefficients.A[6] <= set_p_coefficients; a6; need_write);
            store_coeff!(ws.P_Coefficients.A[7] <= set_p_coefficients; a7; need_write);
            store_coeff!(ws.P_Coefficients.A[8] <= set_p_coefficients; a8; need_write);
            store_coeff!(ws.P_Coefficients.A[9] <= set_p_coefficients; a9; need_write);
            store_coeff!(ws.P_Coefficients.A[10] <= set_p_coefficients; a10; need_write);
            store_coeff!(ws.P_Coefficients.A[11] <= set_p_coefficients; a11; need_write);
            store_coeff!(ws.P_Coefficients.A[12] <= set_p_coefficients; a12; need_write);
            store_coeff!(ws.P_Coefficients.A[13] <= set_p_coefficients; a13; need_write);
            store_coeff!(ws.P_Coefficients.A[14] <= set_p_coefficients; a14; need_write);
            store_coeff!(ws.P_Coefficients.A[15] <= set_p_coefficients; a15; need_write);
        }

        if let Some(set_t_coefficients) = &w.set_t_coefficients {
            store_coeff!(ws.T_Coefficients.F0 <= set_t_coefficients; f0; need_write);
            store_coeff!(ws.T_Coefficients.C[0] <= set_t_coefficients; c1; need_write);
            store_coeff!(ws.T_Coefficients.C[1] <= set_t_coefficients; c2; need_write);
            store_coeff!(ws.T_Coefficients.C[2] <= set_t_coefficients; c3; need_write);
            store_coeff!(ws.T_Coefficients.C[3] <= set_t_coefficients; c4; need_write);
            store_coeff!(ws.T_Coefficients.C[4] <= set_t_coefficients; c5; need_write);
            store_coeff!(ws.T_Coefficients.T0 <= set_t_coefficients; t0; need_write);
        }

        if let Some(set_p_work_range) = &w.set_p_work_range {
            store_coeff!(ws.PWorkRange.minimum <= set_p_work_range; minimum; need_write);
            store_coeff!(ws.PWorkRange.maximum <= set_p_work_range; maximum; need_write);
            store_coeff!(ws.PWorkRange.absolute_maximum <= set_p_work_range; absolute_maximum; need_write);
        }
        if let Some(set_t_work_range) = &w.set_t_work_range {
            store_coeff!(ws.TWorkRange.minimum <= set_t_work_range; minimum; need_write);
            store_coeff!(ws.TWorkRange.maximum <= set_t_work_range; maximum; need_write);
            store_coeff!(ws.TWorkRange.absolute_maximum <= set_t_work_range; absolute_maximum; need_write);
        }
        if let Some(set_tcpu_work_range) = &w.set_tcpu_work_range {
            store_coeff!(ws.TCPUWorkRange.minimum <= set_tcpu_work_range; minimum; need_write);
            store_coeff!(ws.TCPUWorkRange.maximum <= set_tcpu_work_range; maximum; need_write);
            store_coeff!(ws.TCPUWorkRange.absolute_maximum <= set_tcpu_work_range; absolute_maximum; need_write);
        }
        if let Some(set_bat_work_range) = &w.set_bat_work_range {
            store_coeff!(ws.VbatWorkRange.minimum <= set_bat_work_range; minimum; need_write);
            store_coeff!(ws.VbatWorkRange.maximum <= set_bat_work_range; maximum; need_write);
            store_coeff!(ws.VbatWorkRange.absolute_maximum <= set_bat_work_range; absolute_maximum; need_write);
        }

        if let Some(set_calibration_date) = &w.set_calibration_date {
            store_coeff!(ws.calibration_date.Day <= set_calibration_date; day; need_write);
            store_coeff!(ws.calibration_date.Month <= set_calibration_date; month; need_write);
            store_coeff!(ws.calibration_date.Year <= set_calibration_date; year; need_write);
        }

        store_coeff!(ws.PZeroCorrection <= w; set_p_zero_correction; need_write);
        store_coeff!(ws.TZeroCorrection <= w; set_t_zero_correction; need_write);

        if let Some(set_write_config) = &w.set_write_config {
            store_coeff!(ws.writeConfig.BaseInterval_ms <= set_write_config; base_interval_ms; need_write);
            store_coeff!(ws.writeConfig.PWriteDevider <= set_write_config; p_write_devider; need_write);
            store_coeff!(ws.writeConfig.TWriteDevider <= set_write_config; t_write_devider; need_write);
        }

        store_coeff!(ws.startDelay <= w; set_start_delay; need_write);

        if let Some(set_pressure_meassure_units) = w.set_pressure_meassure_units {
            if let Some(mu) = num::FromPrimitive::from_i32(set_pressure_meassure_units) {
                ws.pressureMeassureUnits = mu;
            } else {
                return Err("Invalid measure unit".to_string());
            }
            need_write = true;
        }

        if let Some(set_password) = &w.set_password {
            let newlen = core::cmp::min(set_password.len(), PASSWORD_SIZE);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    set_password.as_ptr(),
                    ts.current_password.as_mut_ptr(),
                    newlen,
                );
            }
            ts.current_password[newlen..].fill(b'\0');
        }

        Ok(need_write)
    })
}
