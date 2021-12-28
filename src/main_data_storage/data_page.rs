use crate::{threads::sensor_processor::FChannel, workmodes::output_storage::OutputStorage};

pub trait DataPage {
    /// записать "шапку"
    fn write_header(&mut self, output: &OutputStorage);

    /// false - место еще доступно
    /// true - место закончилось
    fn push_data(&mut self, result: Option<u32>, channel: FChannel) -> bool;

    /// Финализация пакета
    fn finalise(&mut self);
}
