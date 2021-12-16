use alloc::sync::Arc;
use freertos_rust::{FreeRtosError, Mutex};

use crate::workmodes::output_storage::OutputStorage;

use super::{
    messages::ru_sktbelpa_pressure_self_writer_FlashStatus,
    ru_sktbelpa_pressure_self_writer_Response,
};

pub fn process_requiest(
    req: super::messages::Request,
    mut resp: ru_sktbelpa_pressure_self_writer_Response,
    output: &Arc<Mutex<OutputStorage>>,
) -> Result<ru_sktbelpa_pressure_self_writer_Response, ()> {
    use super::messages::{
        ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER,
        ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID,
        ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION,
        ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS,
        ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR,
    };

    if !(req.device_id == ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID
        || req.device_id == ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER)
    {
        defmt::error!("Protobuf: unknown target device id: 0x{:X}", req.device_id);

        resp.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR;
        return Ok(resp);
    }

    if req.protocol_version != ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION {
        defmt::warn!(
            "Protobuf: unsupported protocol version {}",
            req.protocol_version
        );
        resp.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR;

        return Ok(resp);
    }

    if let Some(writeSettings) = req.write_settings {
        resp.has_getSettings = true;
        match super::process_settings::update_settings(&writeSettings) {
            Ok(need_to_write) => {
                if let Err(e) = super::start_writing_settings(need_to_write) {
                    free_rtos_error(e);
                    resp.Global_status =
                        ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
                }
            }
            Err(e) => {
                defmt::error!("Set settings error: {}", defmt::Debug2Format(&e));
                resp.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
            }
        }
        super::process_settings::fill_settings(&mut resp.getSettings)?;
    }

    if let Some(get_info) = req.get_info {
        resp.has_info = true;
        super::device_info::fill_info(&mut resp.info)?;
    }

    if let Some(change_password) = req.change_password {
        resp.has_changePasswordStatus = true;
        resp.changePasswordStatus.passwordChanged =
            match super::change_password::change_password(&change_password) {
                Err(e) => {
                    defmt::error!("Failed to change password: {}", defmt::Debug2Format(&e));
                    resp.Global_status =
                        ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
                    false
                }
                Ok(need_save) => {
                    if let Err(e) = super::start_writing_settings(need_save) {
                        free_rtos_error(e);
                        resp.Global_status =
                            ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
                        false
                    } else {
                        true
                    }
                }
            };
    }

    if let Some(flash_command) = req.flash_command {
        resp.has_flashStatus = true;

        let mut reset_monitoring_failed = None;
        if let Some(reset_monitoring) = flash_command.reset_monitoring {
            defmt::warn!("Reseting monitoring flags!");
            reset_monitoring_failed = if let Err(e) =
                crate::protobuf::monitoring_over_conditions::reset_monitoring_flags()
            {
                defmt::error!(
                    "Failed to perform flash operation: {}",
                    defmt::Debug2Format(&e)
                );
                resp.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
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
                    resp.Global_status =
                        ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
                }
            }
        }
        fill_flash_state(&mut resp.flashStatus, reset_monitoring_failed)?;
    }

    if let Some(get_output_values) = req.get_output_values {
        resp.has_output = true;
        if let Err(_) = super::output::fill_output(&mut resp.output, &get_output_values, output) {
            resp.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
        }
    }

    Ok(resp)
}

fn fill_flash_state(
    flash_status: &mut ru_sktbelpa_pressure_self_writer_FlashStatus,
    reset_monitoring_failed: Option<bool>,
) -> Result<(), ()> {
    use super::messages as m;

    flash_status.FlashPageSize = crate::main_data_storage::flash_page_size();

    flash_status.status = if let Some(true) = reset_monitoring_failed {
        m::_ru_sktbelpa_pressure_self_writer_FlashStatus_Status_ru_sktbelpa_pressure_self_writer_FlashStatus_Status_ResetMonitoringFailed
    } else {
        m::_ru_sktbelpa_pressure_self_writer_FlashStatus_Status_ru_sktbelpa_pressure_self_writer_FlashStatus_Status_OK
    };

    Ok(())
}

fn free_rtos_error(e: FreeRtosError) {
    defmt::error!("Failed to store settings: {}", defmt::Debug2Format(&e));
}
