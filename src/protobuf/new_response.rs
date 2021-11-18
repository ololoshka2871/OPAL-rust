use freertos_rust::FreeRtosUtils;

use super::ru_sktbelpa_pressure_self_writer_Response;

use super::messages::{
    ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID,
    ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION,
    ru_sktbelpa_pressure_self_writer_STATUS_OK,
};

pub fn new_response(id: u32) -> ru_sktbelpa_pressure_self_writer_Response {
    let mut res = ru_sktbelpa_pressure_self_writer_Response::default();

    res.id = id;
    res.deviceID = ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID;
    res.protocolVersion = ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION;
    res.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_OK;
    res.timestamp = FreeRtosUtils::get_tick_count() as u64;

    res
}
