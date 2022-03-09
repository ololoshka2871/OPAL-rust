use alloc::sync::Arc;
use freertos_rust::{FreeRtosError, Mutex};

use crate::{settings::start_writing_settings, workmodes::output_storage::OutputStorage};

pub fn process_requiest(
    req: super::messages::Request,
    mut resp: super::messages::Response,
    output: &Arc<Mutex<OutputStorage>>,
    cq: &freertos_rust::Queue<crate::threads::sensor_processor::Command>,
) -> Result<super::messages::Response, ()> {
    if !(req.device_id == super::messages::Info::PressureSelfWriterId as u32
        || req.device_id == super::messages::Info::IdDiscover as u32)
    {
        defmt::error!("Protobuf: unknown target device id: 0x{:X}", req.device_id);

        resp.global_status = super::messages::Status::ProtocolError as i32;
        return Ok(resp);
    }

    if req.protocol_version != super::messages::Info::ProtocolVersion as u32 {
        defmt::warn!(
            "Protobuf: unsupported protocol version {}",
            req.protocol_version
        );
        resp.global_status = super::messages::Status::ProtocolError as i32;
        return Ok(resp);
    }

    if let Some(write_settings) = req.write_settings {
        match super::process_settings::update_settings(&write_settings, cq) {
            Ok(need_to_write) => {
                if let Err(e) = start_writing_settings(need_to_write) {
                    free_rtos_error(e);
                    resp.global_status = super::messages::Status::ErrorsInSubcommands as i32;
                }
            }
            Err(e) => {
                defmt::error!("Set settings error: {}", defmt::Debug2Format(&e));
                resp.global_status = super::messages::Status::ErrorsInSubcommands as i32;
            }
        }
        let mut get_settings = super::messages::SettingsResponse::default();
        super::process_settings::fill_settings(&mut get_settings)?;
        resp.get_settings = Some(get_settings);
    }

    if req.get_info.is_some() {
        let mut info = super::messages::InfoResponse::default();
        super::device_info::fill_info(&mut info)?;
        resp.info = Some(info);
    }

    if let Some(change_password) = req.change_password {
        resp.change_password_status = Some(super::messages::ChangePasswordStatus {
            password_changed: match super::change_password::change_password(&change_password) {
                Err(e) => {
                    defmt::error!("Failed to change password: {}", defmt::Debug2Format(&e));
                    resp.global_status = super::messages::Status::ErrorsInSubcommands as i32;
                    false
                }
                Ok(need_save) => {
                    if let Err(e) = start_writing_settings(need_save) {
                        free_rtos_error(e);
                        resp.global_status = super::messages::Status::ErrorsInSubcommands as i32;
                        false
                    } else {
                        true
                    }
                }
            },
        });
    }

    if let Some(flash_command) = req.flash_command {
        let mut flash_status = super::messages::FlashStatus::default();

        let mut reset_monitoring_failed = None;
        if flash_command.reset_monitoring.is_some() {
            defmt::warn!("Reseting monitoring flags!");
            reset_monitoring_failed = if let Err(e) =
                crate::protobuf::monitoring_over_conditions::reset_monitoring_flags()
            {
                defmt::error!(
                    "Failed to perform flash operation: {}",
                    defmt::Debug2Format(&e)
                );
                resp.global_status = super::messages::Status::ErrorsInSubcommands as i32;
                Some(true)
            } else {
                Some(false)
            }
        }
        if let Some(clear_memory) = flash_command.clear_memory {
            if clear_memory {
                defmt::warn!("Start clearing memory!");
                if let Err(e) = crate::main_data_storage::flash_erease() {
                    defmt::error!("Failed to start clear memory: {}", defmt::Debug2Format(&e));
                    resp.global_status = super::messages::Status::ErrorsInSubcommands as i32;
                }
            }
        }
        fill_flash_state(&mut flash_status, reset_monitoring_failed)?;

        resp.flash_status = Some(flash_status);
    }

    if let Some(get_output_values) = req.get_output_values {
        let mut out = super::messages::OutputResponse::default();
        if let Err(_) = super::output::fill_output(&mut out, &get_output_values, output) {
            resp.global_status = super::messages::Status::ErrorsInSubcommands as i32;
        }
        resp.output = Some(out);
    }

    Ok(resp)
}

fn fill_flash_state(
    flash_status: &mut super::messages::FlashStatus,
    reset_monitoring_failed: Option<bool>,
) -> Result<(), ()> {
    flash_status.flash_page_size = crate::main_data_storage::flash_page_size();

    flash_status.status = if let Some(true) = reset_monitoring_failed {
        super::messages::flash_status::Status::ResetMonitoringFailed
    } else if crate::main_data_storage::is_erase_in_progress() {
        super::messages::flash_status::Status::Ereasing
    } else {
        super::messages::flash_status::Status::Ok
    } as i32;

    Ok(())
}

fn free_rtos_error(e: FreeRtosError) {
    defmt::error!("Failed to store settings: {}", defmt::Debug2Format(&e));
}
