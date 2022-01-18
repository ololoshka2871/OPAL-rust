use alloc::sync::Arc;
use freertos_rust::{CurrentTask, Duration, FreeRtosError, Mutex};
use self_recorder_packet::DataBlockPacker;

use crate::{
    sensors::freqmeter::master_counter::{MasterCounter, MasterTimerInfo},
    settings,
    threads::sensor_processor::FChannel,
    workmodes::output_storage::OutputStorage,
};

use super::{
    data_page::DataPage,
    write_controller::{self, WriteController},
};

pub struct CpuFlashDiffWriter {
    master_counter_info: MasterTimerInfo,
    next_page_number: u32,
    crc_calc: Arc<Mutex<stm32l4xx_hal::crc::Crc>>,
    page_aqured: bool,
}

pub struct DataBlock {
    packer: DataBlockPacker,
    counter: usize,
    prevs: [u32; 2],
}

impl DataPage for DataBlock {
    fn write_header(&mut self, output: &OutputStorage) {
        defmt::debug!("DataPage::write_header");
        let h = &mut self.packer.header;

        h.targets = output.targets;
        h.t_cpu = output.t_cpu;
        h.v_bat = output.vbat;
    }

    fn push_data(&mut self, result: Option<u32>, channel: FChannel) -> bool {
        //defmt::debug!("DataPage::push_data(result={}, ch={})", result, channel);
        let v = if let Some(r) = result {
            r - self.prevs[channel as usize]
        } else {
            self.prevs[channel as usize]
        };
        self.counter += 1;
        match self.packer.push_val(v) {
            self_recorder_packet::PushResult::Success => false,
            self_recorder_packet::PushResult::Full => true,
            self_recorder_packet::PushResult::Overflow => false,
            self_recorder_packet::PushResult::Finished => unreachable!(),
        }
    }

    fn finalise(&mut self) {
        //defmt::debug!("DataPage::finalise()");
    }
}

impl CpuFlashDiffWriter {
    pub fn new(crc_calc: Arc<Mutex<stm32l4xx_hal::crc::Crc>>) -> Self {
        let mut master_counter_info = MasterCounter::acquire();
        master_counter_info.want_start();

        Self {
            master_counter_info,
            next_page_number: 0,
            crc_calc,
            page_aqured: false,
        }
    }
}

impl WriteController<DataBlock> for CpuFlashDiffWriter {
    fn try_create_new_page(&mut self) -> Result<DataBlock, FreeRtosError> {
        if !self.page_aqured {
            defmt::info!("Aquaering Data Block");

            let (base_interval_ms, interleave_ratio) =
                match settings::settings_action::<_, _, _, ()>(Duration::ms(10), |(settings, _)| {
                    Ok((
                        settings.writeConfig.BaseInterval_ms,
                        [
                            settings.writeConfig.PWriteDevider,
                            settings.writeConfig.TWriteDevider,
                        ],
                    ))
                }) {
                    Ok(r) => r,
                    Err(settings::SettingActionError::AccessError(e)) => return Err(e),
                    _ => unreachable!(),
                };

            let res = DataBlock {
                packer: DataBlockPacker::builder()
                    .set_ids(
                        self.next_page_number.checked_sub(1).unwrap_or_default(),
                        self.next_page_number,
                    )
                    .set_size(crate::main_data_storage::flash_page_size() as usize)
                    .set_timestamp(self.master_counter_info.uptime_ms())
                    .set_write_cfg(base_interval_ms, interleave_ratio)
                    .build(),
                counter: 0,
                prevs: [0, 0],
            };

            self.next_page_number += 1;
            self.page_aqured = true;

            Ok(res)
        } else {
            Err(freertos_rust::FreeRtosError::OutOfMemory)
        }
    }

    fn write(&mut self, page: DataBlock) -> write_controller::PageWriteResult {
        let id = page.packer.header.this_block_id;
        let input_count = page.counter;

        self.page_aqured = false;
        if let Some(data) = page.packer.to_result_full(|data| {
            self.crc_calc
                .lock(Duration::infinite())
                .map(|mut crc_guard| {
                    crc_guard.reset();
                    crc_guard.feed(data);
                    crc_guard.result()
                })
                .unwrap_or_default()
        }) {
            defmt::info!(
                "Write page {}, {} values ({} bytes) -> {}",
                id,
                input_count,
                input_count * core::mem::size_of::<u32>(),
                data.len()
            );

            // TODO: !write!

            write_controller::PageWriteResult::Succes(id)
        } else {
            defmt::error!("Page {} generation failed!", id);
            write_controller::PageWriteResult::Fail(id)
        }
    }
}
