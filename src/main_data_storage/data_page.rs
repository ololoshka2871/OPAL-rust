use crate::{threads::sensor_processor::FChannel, workmodes::output_storage::OutputStorage};

pub trait DataPage {
    fn write_header(&mut self, output: &OutputStorage);
    fn push_data(&mut self, result: Option<u32>, channel: FChannel) -> bool;
    fn finalise(&mut self);
}
