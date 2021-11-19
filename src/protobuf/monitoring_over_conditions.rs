use freertos_rust::Duration;

use crate::settings::SettingActionError;

pub fn reset_monitoring_flags() -> Result<bool, SettingActionError<u32>> {
    crate::settings::settings_action(Duration::ms(1), |(ws, _)| {
        let need_store = Ok(ws.monitoring.is_set());
        ws.monitoring = crate::settings::app_settings::Monitoring::default();

        need_store
    })
}
