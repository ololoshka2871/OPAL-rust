use super::{
    process_settings::{fill_settings, update_settings},
    ru_sktbelpa_pressure_self_writer_Request, ru_sktbelpa_pressure_self_writer_Response,
};

pub use super::messages::{
    ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER,
    ru_sktbelpa_pressure_self_writer_INFO_MAGICK,
    ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID,
    ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION,
    ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS,
    ru_sktbelpa_pressure_self_writer_STATUS_OK,
    ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR, P_COEFFS_COUNT, T_COEFFS_COUNT,
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
        if let Err(e) = update_settings(&req.writeSettings) {
            defmt::error!("Set settings error: {}", defmt::Debug2Format(&e));
            resp.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
        }
        fill_settings(&mut resp.getSettings)?;
    }

    Ok(resp)
}
