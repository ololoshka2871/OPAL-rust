use core::{cmp::min, ops::Sub};

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

#[derive(PartialEq, Eq)]
pub enum Ordering {
    Greater,
    Less,
}

pub struct OverMonitor<const O: Ordering>(u32);

impl<const O: Ordering> OverMonitor<O> {
    pub fn check<T: Into<f32>>(&mut self, current: T, limit: f32) -> bool {
        if limit.is_nan() {
            return false;
        }

        let cmp = if O == Ordering::Greater {
            current.into() > limit
        } else {
            current.into() > limit
        };

        if cmp {
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

pub(crate) fn read_settings<F, R>(f: F) -> Result<R, freertos_rust::FreeRtosError>
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

pub fn recorder_start_delay() -> Result<u32, freertos_rust::FreeRtosError> {
    read_settings(|(ws, _)| Ok(ws.startDelay))
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

pub fn calc_pressure(fp: f64, output: &mut OutputStorage) {
    static mut P_OVER_MONITOR: OverMonitor<{ Ordering::Greater }> = OverMonitor(0);

    let ft = output.values[FChannel::Temperature as usize];

    if let Ok((t, overpress_rised)) = read_settings(|(ws, _)| {
        let pressure = calc_p(fp, ft, &ws.P_Coefficients, ws.T_enabled);

        let overpress =
            unsafe { P_OVER_MONITOR.check(pressure as f32, ws.PWorkRange.absolute_maximum) };

        let overpress_rised = overpress && !ws.monitoring.Ovarpress;
        if overpress {
            ws.monitoring.Ovarheat = true;
        }

        let pressure = wrap_mu(pressure, ws.pressureMeassureUnits);

        let pressure_fixed = pressure + ws.PZeroCorrection as f64;

        //defmt::trace!("Pressure {} ({}Hz)", pressure, fp);

        Ok((pressure_fixed, overpress_rised))
    }) {
        output.values[FChannel::Pressure as usize] = Some(t);

        if overpress_rised {
            defmt::error!("Pressure: Overpress detected!");
            let _ = crate::settings::settings_save(Duration::ms(50))
                .map_err(|_| unsafe { P_OVER_MONITOR.mast_retry() });
        }
    }
}

pub fn calc_temperature(f: f64, output: &mut OutputStorage) {
    static mut T_OVER_MONITOR: OverMonitor<{ Ordering::Greater }> = OverMonitor(0);

    if let Ok((t, overheat_rised)) = read_settings(|(ws, _)| {
        let temperature = calc_t(f, &ws.T_Coefficients);
        let overheat =
            unsafe { T_OVER_MONITOR.check(temperature as f32, ws.TWorkRange.absolute_maximum) };

        let overheat_rised = overheat && !ws.monitoring.Ovarheat;
        if overheat {
            ws.monitoring.Ovarheat = true;
        }

        let temperature_fixed = temperature + ws.TZeroCorrection as f64;

        //defmt::trace!("Temperature {} ({}Hz)", temperature, f);

        Ok((temperature_fixed, overheat_rised))
    }) {
        output.values[FChannel::Temperature as usize] = Some(t);

        if overheat_rised {
            defmt::error!("Temperature: Overheat detected!");
            let _ = crate::settings::settings_save(Duration::ms(50))
                .map_err(|_| unsafe { T_OVER_MONITOR.mast_retry() });
        }
    }
}

/*
pub fn unwrap_result(wraped: bool, prev_res: Option<u32>, mut result: u32) -> u32 {
    if wraped {
        // анализ результата: Если он меньше чем прошлый на величину около 0x10000 +/- 500
        // то делаем поправку
        if let Some(oldres) = prev_res {
            let diff = (oldres as i64 - result as i64).abs();
            if diff < 0x10000 + 500 && diff > 0x10000 - 500 {
                result -= 0x10000;
            }
        }
    }
    result
}
*/

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

fn wrap_mu(p: f64, mu: crate::settings::app_settings::PressureMeassureUnits) -> f64 {
    use crate::settings::app_settings::PressureMeassureUnits;

    let multiplier = match mu {
        PressureMeassureUnits::INVALID_ZERO => panic!(),
        PressureMeassureUnits::Pa => 100000.0,
        PressureMeassureUnits::Bar => 1.0,
        PressureMeassureUnits::At => 1.0197162,
        PressureMeassureUnits::mmH20 => 10197.162,
        PressureMeassureUnits::mHg => 750.06158 / 1000.0,
        PressureMeassureUnits::Atm => 0.98692327,
        PressureMeassureUnits::PSI => 14.5,
    };

    p * multiplier
}

pub fn process_t_cpu(
    output: &Mutex<OutputStorage>,
    current_period_ticks: u32,
    celsius_degree: f32,
    raw: u16,
    sys_clk: Hertz,
) -> (bool, Option<u32>) {
    static mut TCPU_OVER_MONITOR: OverMonitor<{ Ordering::Greater }> = OverMonitor(0);

    //defmt::trace!("CPU Temperature {} ({})", celsius_degree, raw);

    if let Ok((overheat_rised, t_mt)) = read_settings(|(ws, _)| {
        let overheat =
            unsafe { TCPU_OVER_MONITOR.check(celsius_degree, ws.TCPUWorkRange.absolute_maximum) };

        let overheat_rised = overheat && !ws.monitoring.CPUOvarheat;
        if overheat {
            ws.monitoring.CPUOvarheat = true;
        }

        Ok((overheat_rised, ws.TMesureTime_ms))
    }) {
        output
            .lock(Duration::infinite())
            .map(|mut guard| {
                guard.t_cpu = celsius_degree;
                guard.t_cpu_adc = raw;
            })
            .unwrap();

        if overheat_rised {
            defmt::error!("CPU Temperature: Overheat detected!");

            let _ = crate::settings::settings_save(Duration::ms(50))
                .map_err(|_| unsafe { TCPU_OVER_MONITOR.mast_retry() });
        }

        return (true, analog_period(current_period_ticks, t_mt, sys_clk));
    }
    return (true, None);
}

pub fn process_vbat(
    output: &Mutex<OutputStorage>,
    current_period_ticks: u32,
    vbat_input_mv: u16,
    raw: u16,
    sys_clk: Hertz,
) -> (bool, Option<u32>) {
    static mut VBAT_OVER_MONITOR: OverMonitor<{ Ordering::Greater }> = OverMonitor(0);
    static mut VBAT_UNDER_MONITOR: OverMonitor<{ Ordering::Less }> = OverMonitor(0);

    if let Ok((vbat, overvoltage_raised, undervoltage_detected, mt)) = read_settings(|(ws, _)| {
        let v_bat = vbat_input_mv as f32 / 1000.0
            * (crate::config::VBAT_DEVIDER_R1 + crate::config::VBAT_DEVIDER_R2)
            / crate::config::VBAT_DEVIDER_R2;

        let overvoltage =
            unsafe { VBAT_OVER_MONITOR.check(v_bat, ws.VbatWorkRange.absolute_maximum) };
        let overvoltage_raised = overvoltage && !ws.monitoring.OverPower;
        if overvoltage {
            ws.monitoring.OverPower = true;
        }

        let undervoltage = unsafe { VBAT_UNDER_MONITOR.check(v_bat, ws.VbatWorkRange.minimum) };

        Ok((
            v_bat,
            overvoltage_raised,
            undervoltage,
            min(ws.PMesureTime_ms, ws.TMesureTime_ms),
        ))
    }) {
        //defmt::trace!("Vbat {} mv ({} mv / {})", vbat, vbat_input_mv, raw);

        output
            .lock(Duration::infinite())
            .map(|mut guard| {
                guard.vbat = vbat;
                guard.vbat_adc = raw;
            })
            .unwrap();

        if overvoltage_raised {
            defmt::error!("Vbat overvoltage detected!");

            let _ = crate::settings::settings_save(Duration::ms(50))
                .map_err(|_| unsafe { VBAT_OVER_MONITOR.mast_retry() });
        }
        return (
            !undervoltage_detected,
            analog_period(current_period_ticks, mt, sys_clk),
        );
    }
    return (true, None);
}

fn analog_period(cutternt_t: u32, t: u32, sys_clk: Hertz) -> Option<u32> {
    let new_period_ticks =
        crate::workmodes::common::to_real_period(Duration::ms(t), sys_clk).to_ms();

    if new_period_ticks != cutternt_t {
        Some(new_period_ticks)
    } else {
        None
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

/// Вечный сон
pub fn halt_cpu() -> ! {
    cortex_m::interrupt::free(|_| cortex_m::asm::wfi());
    loop {}
}
