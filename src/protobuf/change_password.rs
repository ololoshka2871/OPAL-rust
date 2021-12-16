use alloc::string::{String, ToString};
use freertos_rust::Duration;

use crate::settings::SettingActionError;

use super::PASSWORD_SIZE;

pub fn change_password(
    cmd: &super::messages::ChangePassword,
) -> Result<bool, SettingActionError<String>> {
    let trimmed_pass = &cmd.new_password[..PASSWORD_SIZE].as_bytes();
    crate::settings::settings_action(Duration::ms(1), |(ws, ts)| {
        if ws.password != ts.current_password {
            Err("Invalid password".to_string())
        } else if ts.current_password == *trimmed_pass {
            return Ok(false);
        } else {
            ws.password.copy_from_slice(trimmed_pass);
            ts.current_password.copy_from_slice(trimmed_pass);
            return Ok(true);
        }
    })
}
