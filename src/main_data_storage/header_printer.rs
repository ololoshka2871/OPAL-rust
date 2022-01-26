use defmt::{write, Format, Formatter};
use self_recorder_packet::DataPacketHeader;

pub struct HeaderPrinter<'a>(pub &'a DataPacketHeader);

impl<'a> Format for HeaderPrinter<'a> {
    fn format(&self, fmt: Formatter) {
        write!(
            fmt,
            r#"Block {} (prev {}): {{
    timestamp: {},
    targets: {},
    base_interval_ms: {},
    interleave_ratio: {},
    t_cpu: {},
    v_bat: {}
}}"#,
            self.0.this_block_id,
            self.0.prev_block_id,
            self.0.timestamp,
            self.0.targets,
            self.0.base_interval_ms,
            self.0.interleave_ratio,
            self.0.t_cpu,
            self.0.v_bat,
        );
    }
}
