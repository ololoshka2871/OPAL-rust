use nanopb_rs::pb_msgdesc_t;

use super::messages;

impl messages::ru_sktbelpa_pressure_self_writer_Request {
    pub fn fields() -> &'static pb_msgdesc_t {
        unsafe { &messages::ru_sktbelpa_pressure_self_writer_Request_msg }
    }
}

impl messages::ru_sktbelpa_pressure_self_writer_Response {
    pub fn fields() -> &'static pb_msgdesc_t {
        unsafe { &messages::ru_sktbelpa_pressure_self_writer_Response_msg }
    }
}

impl messages::ru_sktbelpa_pressure_self_writer_PCoefficients {
    pub fn fields() -> &'static pb_msgdesc_t {
        unsafe { &messages::ru_sktbelpa_pressure_self_writer_PCoefficients_msg }
    }
}

impl messages::ru_sktbelpa_pressure_self_writer_T5Coefficients {
    pub fn fields() -> &'static pb_msgdesc_t {
        unsafe { &messages::ru_sktbelpa_pressure_self_writer_T5Coefficients_msg }
    }
}
