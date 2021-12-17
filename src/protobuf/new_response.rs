use freertos_rust::FreeRtosUtils;

use super::messages::{Info, Response, Status};

pub fn new_response(id: u32) -> Response {
    let mut res = Response::default();

    res.id = id;
    res.device_id = Info::PressureSelfWriterId as u32;
    res.protocol_version = Info::ProtocolVersion as u32;
    res.global_status = Status::Ok as i32;
    res.timestamp = FreeRtosUtils::get_tick_count() as u64;

    res
}
