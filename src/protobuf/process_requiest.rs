use defmt::Debug2Format;
use freertos_rust::FreeRtosError;

use super::{
    change_password::change_password,
    device_info::fill_info,
    messages::ru_sktbelpa_pressure_self_writer_FlashStatus,
    process_settings::{fill_settings, update_settings},
    ru_sktbelpa_pressure_self_writer_Request, ru_sktbelpa_pressure_self_writer_Response,
    start_writing_settings,
};

use super::messages::{
    ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER,
    ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID,
    ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION,
    ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS,
    ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR,
};

pub fn process_requiest(
    req: ru_sktbelpa_pressure_self_writer_Request,
    mut resp: ru_sktbelpa_pressure_self_writer_Response,
) -> Result<ru_sktbelpa_pressure_self_writer_Response, ()> {
    if !(req.deviceID == ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID
        || req.deviceID == ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER)
    {
        defmt::error!("Protobuf: unknown target device id: 0x{:X}", req.deviceID);

        resp.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR;
        return Ok(resp);
    }

    if req.protocolVersion != ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION {
        defmt::warn!(
            "Protobuf: unsupported protocol version {}",
            req.protocolVersion
        );
        resp.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR;

        return Ok(resp);
    }

    if req.has_writeSettings {
        resp.has_getSettings = true;
        match update_settings(&req.writeSettings) {
            Ok(need_to_write) => {
                if let Err(e) = start_writing_settings(need_to_write) {
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
        fill_settings(&mut resp.getSettings)?;
    }

    if req.has_getInfo {
        resp.has_info = true;
        fill_info(&mut resp.info)?;
    }

    if req.has_changePassword {
        resp.has_changePasswordStatus = true;
        resp.changePasswordStatus.passwordChanged = match change_password(&req.changePassword) {
            Err(e) => {
                defmt::error!("Failed to change password: {}", defmt::Debug2Format(&e));
                resp.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
                false
            }
            Ok(need_save) => {
                if let Err(e) = start_writing_settings(need_save) {
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

    if req.has_flashCommand {
        resp.has_flashStatus = true;

        let mut reset_monitoring_failed = None;
        if req.flashCommand.has_ResetMonitoring {
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
        if req.flashCommand.has_ClearMemory && req.flashCommand.ClearMemory == true {
            defmt::warn!("Start clearing memory!");
            if let Err(e) = crate::main_data_storage::flash_erease() {
                defmt::error!("Failed to start clear memory: {}", defmt::Debug2Format(&e));
                resp.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
            }
        }
        fill_flash_state(&mut resp.flashStatus, reset_monitoring_failed)?;
    }

    if req.has_getOutputValues {
        resp.has_output = true;
        super::output::fill_output(&mut resp.output, &req.getOutputValues)?;
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
    defmt::error!("Failed to store settings: {}", Debug2Format(&e));
}
