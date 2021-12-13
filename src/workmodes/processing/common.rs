use core::ops::Sub;

use freertos_rust::{Duration, Mutex};
use stm32l4xx_hal::time::Hertz;

use crate::{
    settings::{app_settings::NonStoreSettings, AppSettings, SettingActionError},
    threads::sensor_processor::FChannel,
    workmodes::{common::HertzExt, output_storage::OutputStorage},
};

pub struct ChannelConfig {
    pub enabled: bool,
}

pub struct OverMonitor(u32);

impl OverMonitor {
    pub fn check(&mut self, current: f64, limit: f32) -> bool {
        if current > limit as f64 {
            if self.0 > crate::config::OVER_LIMIT_COUNT {
                false
            } else if self.0 == crate::config::OVER_LIMIT_COUNT {
                self.0 += 1;
                true
            } else {
                false
            }
        } else {
            self.0 = 0;
            false
        }
    }

    #[allow(dead_code)]
    pub fn mast_retry(&mut self) {
        if self.0 > crate::config::OVER_LIMIT_COUNT {
            self.0 = crate::config::OVER_LIMIT_COUNT;
        }
    }
}

fn read_settings<F, R>(f: F) -> Result<R, freertos_rust::FreeRtosError>
where
    F: FnMut((&mut AppSettings, &mut NonStoreSettings)) -> Result<R, ()>,
{
    crate::settings::settings_action::<_, _, _, ()>(Duration::ms(1), f).map_err(|e| match e {
        SettingActionError::AccessError(e) => e,
        SettingActionError::ActionError(_) => unreachable!(),
    })
}

fn fref_getter() -> Result<f64, freertos_rust::FreeRtosError> {
    read_settings(|(ws, _)| Ok(ws.Fref)).map(|fref| fref as f64)
}

fn mt_getter(ch: FChannel) -> Result<f64, freertos_rust::FreeRtosError> {
    read_settings(|(ws, _)| {
        Ok(match ch {
            FChannel::Pressure => ws.PMesureTime_ms,
            FChannel::Temperature => ws.TMesureTime_ms,
        })
    })
    .map(|fref| fref as f64)
}

pub fn channel_config(ch: FChannel) -> Result<ChannelConfig, freertos_rust::FreeRtosError> {
    read_settings(|(ws, _)| {
        Ok(match ch {
            FChannel::Pressure => ChannelConfig {
                enabled: ws.P_enabled,
            },
            FChannel::Temperature => ChannelConfig {
                enabled: ws.T_enabled,
            },
        })
    })
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

fn mt2guard_ticks(mt: f64, sysclk: &Hertz) -> u32 {
    (sysclk.duration_ms(mt as u32).to_ms() as f32 * crate::config::MEASURE_TIME_TO_GUARD_MULTIPLIER)
        as u32
}

pub fn guard_ticks(ch: FChannel, sysclk: &Hertz) -> Result<u32, freertos_rust::FreeRtosError> {
    let mt = mt_getter(ch)?;
    Ok(mt2guard_ticks(mt, sysclk))
}

pub fn calc_new_target(
    ch: FChannel,
    f: f64,
    sysclk: &Hertz,
) -> Result<(u32, u32), freertos_rust::FreeRtosError> {
    let mt = mt_getter(ch)?;
    let mut new_target = (f * mt / 1000.0f64) as u32;
    if new_target < 1 {
        new_target = 1;
    }

    Ok((new_target, mt2guard_ticks(mt, sysclk)))
}

//---------------------------------------------------------------------------------------

pub fn calc_pressure(fp: f64, output: &Mutex<OutputStorage>) {
    static mut P_OVER_MONITOR: OverMonitor = OverMonitor(0);

    let ft = output
        .lock(Duration::infinite())
        .map(|guard| guard.values[FChannel::Temperature as usize])
        .unwrap();

    if let Ok((t, overpress_rised)) = read_settings(|(ws, _)| {
        let pressure = calc_p(fp, ft, &ws.P_Coefficients, ws.T_enabled);

        let overpress = unsafe { P_OVER_MONITOR.check(pressure, ws.PWorkRange.absolute_maximum) };

        let overpress_rised = overpress && !ws.monitoring.Ovarpress;
        if overpress {
            ws.monitoring.Ovarheat = true;
        }

        //defmt::trace!("Pressure {} ({}Hz)", pressure, fp);

        Ok((pressure, overpress_rised))
    }) {
        output
            .lock(Duration::infinite())
            .map(|mut guard| guard.values[FChannel::Pressure as usize] = Some(t))
            .unwrap();

        if overpress_rised {
            defmt::error!("Pressure: Overpress detected!");
            /*
            let _ = crate::settings::settings_save(Duration::ms(50))
                .map_err(|_| unsafe { P_OVER_MONITOR.mast_retry() });
            */
        }
    }
}

pub fn calc_temperature(f: f64, output: &Mutex<OutputStorage>) {
    static mut T_OVER_MONITOR: OverMonitor = OverMonitor(0);

    if let Ok((t, overheat_rised)) = read_settings(|(ws, _)| {
        let temperature = calc_t(f, &ws.T_Coefficients);
        let overheat = unsafe { T_OVER_MONITOR.check(temperature, ws.TWorkRange.absolute_maximum) };

        let overheat_rised = overheat && !ws.monitoring.Ovarheat;
        if overheat {
            ws.monitoring.Ovarheat = true;
        }

        //defmt::trace!("Temperature {} ({}Hz)", temperature, f);

        Ok((temperature, overheat_rised))
    }) {
        output
            .lock(Duration::infinite())
            .map(|mut guard| guard.values[FChannel::Temperature as usize] = Some(t))
            .unwrap();

        if overheat_rised {
            defmt::error!("Temperature: Overheat detected!");
            /*
            let _ = crate::settings::settings_save(Duration::ms(50))
                .map_err(|_| unsafe { T_OVER_MONITOR.mast_retry() });
            */
        }
    }
}

//-----------------------------------------------------------------------------

pub fn abs_difference<T: Sub<Output = T> + Ord>(x: T, y: T) -> T {
    if x < y {
        y - x
    } else {
        x - y
    }
}

//-----------------------------------------------------------------------------

fn calc_t(f: f64, coeffs: &crate::settings::app_settings::T5Coeffs) -> f64 {
    let temp_f_minus_fp0 = f - coeffs.F0 as f64;
    let mut result = coeffs.T0 as f64;
    let mut mu = temp_f_minus_fp0;

    for i in 0..3 {
        result += mu * coeffs.C[i] as f64;
        mu *= temp_f_minus_fp0;
    }

    result
}

fn calc_p(
    fp: f64,
    ft: Option<f64>,
    coeffs: &crate::settings::app_settings::P16Coeffs,
    t_enabled: bool,
) -> f64 {
    let presf_minus_fp0 = fp - coeffs.Fp0 as f64;
    let ft_minus_ft0 = if !t_enabled || ft.is_none() {
        0.0f64
    } else {
        ft.unwrap() - coeffs.Ft0 as f64
    };

    let a = &coeffs.A;

    let k0 = a[0] as f64
        + ft_minus_ft0 * (a[1] as f64 + ft_minus_ft0 * (a[2] as f64 + ft_minus_ft0 * a[12] as f64));
    let k1 = a[3] as f64
        + ft_minus_ft0 * (a[5] as f64 + ft_minus_ft0 * (a[7] as f64 + ft_minus_ft0 * a[13] as f64));
    let k2 = a[4] as f64
        + ft_minus_ft0 * (a[6] as f64 + ft_minus_ft0 * (a[8] as f64 + ft_minus_ft0 * a[14] as f64));
    let k3 = a[9] as f64
        + ft_minus_ft0
            * (a[10] as f64 + ft_minus_ft0 * (a[11] as f64 + ft_minus_ft0 * a[15] as f64));

    k0 + presf_minus_fp0 * (k1 + presf_minus_fp0 * (k2 + presf_minus_fp0 * k3))
}
