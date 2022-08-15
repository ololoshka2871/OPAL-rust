pub(crate) mod app_settings;
mod flash_rw_polcy;
mod store_async;

use core::ops::DerefMut;

use lazy_static::lazy_static;

use alloc::sync::Arc;
pub(crate) use app_settings::AppSettings;
use flash_settings_rs::SettingsManager;

use self::{app_settings::NonStoreSettings, flash_rw_polcy::Placeholder};
use flash_rw_polcy::FlasRWPolcy;
use freertos_rust::{Duration, DurationTicks, FreeRtosError, Mutex};

pub use store_async::start_writing_settings;

static DEFAULT_SETTINGS: AppSettings = AppSettings { Delay: 0 };

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
    static ref SETTINGS: Mutex<Option<SettingsManagerType>> = crate::support::new_global_mutex();
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
            NonStoreSettings {},
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

fn settings_save<D>(duration: D) -> Result<(), FreeRtosError>
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
