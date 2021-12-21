use super::{data_page::DataPage, write_controller::WriteController};

pub struct TestWriter;

pub struct TestDataPage;

impl DataPage for TestDataPage {
    fn write_header(&mut self, output: &crate::workmodes::output_storage::OutputStorage) {
        todo!()
    }

    fn push_data(
        &mut self,
        result: Option<u32>,
        channel: crate::threads::sensor_processor::FChannel,
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

    fn start_write(&mut self, page: TestDataPage) -> super::write_controller::PageWriteResult {
        todo!()
    }
}
