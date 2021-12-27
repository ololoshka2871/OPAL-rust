#![allow(dead_code)]

use super::{data_page::DataPage, write_controller::WriteController};

pub struct TestWriter;

pub struct TestDataPage;

impl DataPage for TestDataPage {
    fn write_header(&mut self, _output: &crate::workmodes::output_storage::OutputStorage) {
        todo!()
    }

    fn push_data(
        &mut self,
        _result: Option<u32>,
        _channel: crate::threads::sensor_processor::FChannel,
    ) -> bool {
        todo!()
    }

    fn finalise(&mut self) {
        todo!()
    }
}

impl WriteController<TestDataPage> for TestWriter {
    fn new_page(&mut self) -> Result<TestDataPage, freertos_rust::FreeRtosError> {
        todo!()
    }

    fn start_write(&mut self, _page: TestDataPage) -> super::write_controller::PageWriteResult {
        todo!()
    }
}
