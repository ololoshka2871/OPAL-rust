mod app_settings;
mod flash_rw_polcy;

use core::ops::DerefMut;

use lazy_static::lazy_static;

use alloc::sync::Arc;
pub(crate) use app_settings::AppSettings;
use flash_settings_rs::SettingsManager;

use flash_rw_polcy::FlasRWPolcy;
use freertos_rust::{Duration, DurationTicks, FreeRtosError, Mutex};

use self::flash_rw_polcy::Placeholder;

static DEFAULT_SETTINGS: AppSettings = AppSettings {
    serial: 0,
    pmesure_time_ms: 1000,
    tmesure_time_ms: 1000,

    fref: 16000000,

    p_enabled: true,
    t_enabled: true,

    pcoefficients: app_settings::P16Coeffs {
        Fp0: 0.0,
        Ft0: 0.0,
        A: [0.0f32; 16],
    },
    tcoefficients: app_settings::T5Coeffs {
        F0: 0.0,
        T0: 0.0,
        C: [0.0f32; 5],
    },
};

pub(crate) type SettingsManagerType =
    SettingsManager<AppSettings, stm32l4xx_hal::traits::flash::Error, FlasRWPolcy>;

#[link_section = ".settings.app"]
static SETTINGS_PLACEHOLDER: Placeholder<AppSettings> =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<Placeholder<AppSettings>>()]) };

lazy_static! {
    static ref SETTINGS: Mutex<Option<SettingsManagerType>> = Mutex::new(None).unwrap();
}

pub(crate) fn init(flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>) {
    defmt::trace!("Init settings");
    if let Ok(mut guard) = SETTINGS.lock(Duration::infinite()) {
        guard.replace(SettingsManager::<
            AppSettings,
            stm32l4xx_hal::traits::flash::Error,
            FlasRWPolcy,
        >::new(
            &DEFAULT_SETTINGS,
            FlasRWPolcy::create(&SETTINGS_PLACEHOLDER, flash),
        ));
    } else {
        panic!("Failed to init settings");
    }
}

pub(crate) fn settings_action<D, F>(duration: D, f: F) -> Result<(), FreeRtosError>
where
    F: Fn(&mut AppSettings),
    D: DurationTicks,
{
    let mut guard = SETTINGS.lock(duration)?;
    if let Some(manager) = guard.deref_mut() {
        f(manager.ref_mut());
        Ok(())
    } else {
        Err(FreeRtosError::OutOfMemory)
    }
}

pub(crate) fn settings_restore<D>(duration: D) -> Result<(), FreeRtosError>
where
    D: DurationTicks,
{
    let mut guard = SETTINGS.lock(duration)?;
    if let Some(manager) = guard.deref_mut() {
        if manager.load().is_err() {
            return Err(FreeRtosError::OutOfMemory);
        }
    }
    Ok(())
}

pub(crate) fn settings_save<D>(duration: D) -> Result<(), FreeRtosError>
where
    D: DurationTicks,
{
    let mut guard = SETTINGS.lock(duration)?;
    if let Some(manager) = guard.deref_mut() {
        if manager.save().is_err() {
            return Err(FreeRtosError::OutOfMemory);
        }
    }
    Ok(())
}
