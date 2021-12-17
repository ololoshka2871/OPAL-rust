mod change_password;
mod device_info;
mod encode_md_message;
mod md;
mod message_body;
mod messages;
mod monitoring_over_conditions;
mod new_response;
mod output;
mod process_requiest;
mod process_settings;
mod stream;

use freertos_rust::{Duration, FreeRtosError};

pub use encode_md_message::encode_md_message;
pub use md::recive_md_header;
pub use message_body::recive_message_body;
pub use new_response::new_response;
pub use process_requiest::process_requiest;
pub use stream::Stream;

pub use messages::{Response, PASSWORD_SIZE, P_COEFFS_COUNT, T_COEFFS_COUNT};

pub fn start_writing_settings(realy_write: bool) -> Result<(), FreeRtosError> {
    use freertos_rust::{Task, TaskPriority};
    if !realy_write {
        defmt::warn!("Save settings skipped...");
        return Ok(());
    }

    defmt::warn!("Save settings rquested...");

    Task::new()
        .name("SS")
        .stack_size(384)
        .priority(TaskPriority(1))
        .start(move |_| {
            if let Err(e) = crate::settings::settings_save(Duration::infinite()) {
                defmt::error!("Failed to store settings: {}", defmt::Debug2Format(&e));
            }
        })
        .map(|_| ())?;

    Ok(())
}
