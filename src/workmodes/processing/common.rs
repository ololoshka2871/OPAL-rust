use core::ops::Sub;

use freertos_rust::Duration;

use crate::{settings::SettingActionError, threads::sensor_processor::FChannel};

fn fref_getter() -> Result<f64, freertos_rust::FreeRtosError> {
    crate::settings::settings_action::<_, _, _, ()>(Duration::ms(1), |(ws, _)| Ok(ws.Fref))
        .map_err(|e| match e {
            SettingActionError::AccessError(e) => e,
            SettingActionError::ActionError(_) => unreachable!(),
        })
        .map(|fref| fref as f64)
}

fn mt_getter(ch: FChannel) -> Result<f64, freertos_rust::FreeRtosError> {
    crate::settings::settings_action::<_, _, _, ()>(Duration::ms(1), |(ws, _)| {
        Ok(match ch {
            FChannel::Pressure => ws.PMesureTime_ms,
            FChannel::Temperature => ws.TMesureTime_ms,
        })
    })
    .map_err(|e| match e {
        SettingActionError::AccessError(e) => e,
        SettingActionError::ActionError(_) => unreachable!(),
    })
    .map(|mt| mt as f64)
}

pub fn calc_freq(
    fref_multiplier: f64,
    target: u32,
    diff: u32,
) -> Result<f64, freertos_rust::FreeRtosError> {
    let fref = fref_multiplier * fref_getter()?;
    let f = fref * target as f64 / diff as f64;

    Ok(f)
}

pub fn calc_new_target(ch: FChannel, f: f64) -> Result<u32, freertos_rust::FreeRtosError> {
    let mt = mt_getter(ch)?;
    let mut new_target = (f * mt / 1000.0f64) as u32;
    if new_target < 1 {
        new_target = 1;
    }

    Ok(new_target)
}

pub fn abs_difference<T: Sub<Output = T> + Ord>(x: T, y: T) -> T {
    if x < y {
        y - x
    } else {
        x - y
    }
}
