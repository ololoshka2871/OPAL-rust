pub(crate) mod app_settings;
mod flash_rw_polcy;

use core::ops::DerefMut;

use lazy_static::lazy_static;

use alloc::sync::Arc;
pub(crate) use app_settings::AppSettings;
use flash_settings_rs::SettingsManager;

use flash_rw_polcy::FlasRWPolcy;
use freertos_rust::{Duration, DurationTicks, FreeRtosError, Mutex};
use my_proc_macro::{build_day, build_month, build_year};

use self::{app_settings::NonStoreSettings, flash_rw_polcy::Placeholder};

static DEFAULT_SETTINGS: AppSettings = AppSettings {
    Serial: 0,
    PMesureTime_ms: 1000,
    TMesureTime_ms: 1000,

    Fref: 16000000,

    P_enabled: true,
    T_enabled: true,
    TCPUEnabled: true,
    VBatEnable: true,

    P_Coefficients: app_settings::P16Coeffs {
        Fp0: 0.0,
        Ft0: 0.0,
        A: [
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
    },
    T_Coefficients: app_settings::T5Coeffs {
        F0: 0.0,
        T0: 0.0,
        C: [1.0, 0.0, 0.0, 0.0, 0.0],
    },

    PWorkRange: app_settings::WorkRange {
        minimum: 0.0,
        maximum: 100.0,
        absolute_maximum: 120.0,
    },

    TWorkRange: app_settings::WorkRange {
        minimum: -50.0,
        maximum: 120.0,
        absolute_maximum: 125.0,
    },

    TCPUWorkRange: app_settings::WorkRange {
        minimum: -50.0,
        maximum: 120.0,
        absolute_maximum: 127.0,
    },

    VbatWorkRange: app_settings::WorkRange {
        minimum: 1.5,
        maximum: 10.0,
        absolute_maximum: 12.0,
    },

    calibration_date: app_settings::CalibrationDate {
        Day: build_day!(),
        Month: build_month!(),
        Year: build_year!(),
    },

    PZeroCorrection: 0.0,
    TZeroCorrection: 0.0,

    writeConfig: app_settings::WriteConfig {
        BaseInterval_ms: 1000,
        PWriteDevider: 1,
        TWriteDevider: 1,
    },

    startDelay: 0,

    pressureMeassureUnits: app_settings::PressureMeassureUnits::At,

    password: *b"_PASSWORD_",

    monitoring: app_settings::Monitoring {
        Ovarpress: false,
        Ovarheat: false,
        CPUOvarheat: false,
        OverPower: false,
    },
};

pub(crate) type SettingsManagerType = SettingsManager<
    AppSettings,
    NonStoreSettings,
    stm32l4xx_hal::traits::flash::Error,
    FlasRWPolcy,
>;

#[link_section = ".settings.app"]
static SETTINGS_PLACEHOLDER: Placeholder<AppSettings> =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<Placeholder<AppSettings>>()]) };

lazy_static! {
    static ref SETTINGS: Mutex<Option<SettingsManagerType>> = Mutex::new(None).unwrap();
}

#[derive(Debug)]
pub enum SettingActionError<T: core::fmt::Debug> {
    AccessError(FreeRtosError),
    ActionError(T),
}

pub(crate) fn init(
    flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>,
    crc: Arc<Mutex<stm32l4xx_hal::crc::Crc>>,
) {
    defmt::trace!("Init settings");
    if let Ok(mut guard) = SETTINGS.lock(Duration::infinite()) {
        guard.replace(SettingsManager::<
            AppSettings,
            NonStoreSettings,
            stm32l4xx_hal::traits::flash::Error,
            FlasRWPolcy,
        >::new(
            &DEFAULT_SETTINGS,
            FlasRWPolcy::create(&SETTINGS_PLACEHOLDER, flash, crc),
            NonStoreSettings {
                current_password: [0u8; 10],
            },
        ));
    } else {
        panic!("Failed to init settings");
    }
}

pub(crate) fn settings_action<D, F, R, T>(duration: D, mut f: F) -> Result<R, SettingActionError<T>>
where
    F: FnMut((&mut AppSettings, &mut NonStoreSettings)) -> Result<R, T>,
    D: DurationTicks,
    T: core::fmt::Debug,
{
    let mut guard = SETTINGS
        .lock(duration)
        .map_err(|e| SettingActionError::AccessError(e))?;
    if let Some(manager) = guard.deref_mut() {
        f(manager.ref_mut()).map_err(|e| SettingActionError::ActionError(e))
    } else {
        Err(SettingActionError::AccessError(FreeRtosError::OutOfMemory))
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
