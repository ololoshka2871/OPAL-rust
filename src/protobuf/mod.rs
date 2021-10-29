#![allow(dead_code)]

mod messages;

pub use messages::{
    ru_sktbelpa_pressure_self_writer_Request, ru_sktbelpa_pressure_self_writer_Response,
};
pub use nanopb_rs::pb_msgdesc_t;

impl ru_sktbelpa_pressure_self_writer_Request {
    pub fn fields() -> &'static pb_msgdesc_t {
        unsafe { &messages::ru_sktbelpa_pressure_self_writer_Request_msg }
    }
}

impl ru_sktbelpa_pressure_self_writer_Response {
    pub fn fields() -> &'static pb_msgdesc_t {
        unsafe { &messages::ru_sktbelpa_pressure_self_writer_Response_msg }
    }
}
