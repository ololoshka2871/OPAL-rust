#![allow(dead_code)]

use super::{data_page::DataPage, write_controller::WriteController};

#[derive(Default)]
pub struct TestWriter {
    page: u32,
}

#[derive(Default)]
pub struct TestDataPage {
    page_number: u32,
    counter: u8,
}

impl DataPage for TestDataPage {
    fn write_header(&mut self, _output: &crate::workmodes::output_storage::OutputStorage) {
        defmt::debug!("write_header()");
    }

    fn push_data(
        &mut self,
        result: Option<u32>,
        channel: crate::threads::sensor_processor::FChannel,
    ) -> bool {
        defmt::debug!("push_data(res={}, ch={})", result, channel);
        self.counter += 1;
        self.counter > 10
    }

    fn finalise(&mut self) {
        defmt::debug!("finalise()");
    }
}

impl WriteController<TestDataPage> for TestWriter {
    fn new_page(&mut self) -> Result<TestDataPage, freertos_rust::FreeRtosError> {
        defmt::debug!("new_page()");
        let res = Ok(TestDataPage {
            page_number: self.page,
            ..Default::default()
        });
        self.page += 1;

        res
    }

    fn write(&mut self, page: TestDataPage) -> super::write_controller::PageWriteResult {
        defmt::debug!("start_write(page={})", page.page_number);
        super::write_controller::PageWriteResult::Succes(page.page_number)
    }
}
