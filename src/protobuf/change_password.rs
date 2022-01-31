use alloc::string::{String, ToString};
use freertos_rust::Duration;

use crate::settings::SettingActionError;

use super::PASSWORD_SIZE;

pub fn change_password(
    cmd: &super::messages::ChangePassword,
) -> Result<bool, SettingActionError<String>> {
    let mut pass = [0u8; PASSWORD_SIZE];
    unsafe {
        core::ptr::copy_nonoverlapping(
            cmd.new_password.as_ptr(),
            pass.as_mut_ptr(),
            core::cmp::min(cmd.new_password.len(), PASSWORD_SIZE),
        );
    }
    crate::settings::settings_action(Duration::ms(1), |(ws, ts)| {
        if ws.password != ts.current_password {
            Err("Invalid password".to_string())
        } else if ts.current_password == pass {
            return Ok(false);
        } else {
            ws.password.copy_from_slice(&pass);
            ts.current_password.copy_from_slice(&pass);
            return Ok(true);
        }
    })
}
