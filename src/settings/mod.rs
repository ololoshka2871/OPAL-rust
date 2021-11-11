mod app_settings;
mod flash_rw_polcy;

pub(crate) use app_settings::AppSettings;
use flash_settings_rs::SettingsManager;

use flash_rw_polcy::FlasRWPolcy;

#[link_section = ".settings.app"]
static SETTINGS_PLACEHOLDER: AppSettings = AppSettings {};

pub(crate) static mut SETTINGS: Option<
    SettingsManager<AppSettings, stm32l4xx_hal::traits::flash::Error, FlasRWPolcy>,
> = None;

pub(crate) fn init(flash: &mut stm32l4xx_hal::flash::Parts) {
    defmt::trace!("Init settings");
    unsafe {
        SETTINGS = Some(SettingsManager::<
            AppSettings,
            stm32l4xx_hal::traits::flash::Error,
            FlasRWPolcy,
        >::new(FlasRWPolcy::new(
            flash,
            &SETTINGS_PLACEHOLDER,
        )))
    };
}
