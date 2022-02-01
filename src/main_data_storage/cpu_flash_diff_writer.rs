use alloc::sync::Arc;
use freertos_rust::{Duration, FreeRtosError, Mutex};
use self_recorder_packet::DataBlockPacker;

use crate::{
    main_data_storage::PageAccessor,
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
    fref_mul: f32,
    page_aqured: bool,
}

pub struct DataBlock {
    packer: DataBlockPacker,
    counter: usize,
    dest_page: u32,
    prevs: [u32; 2],
}

impl DataPage for DataBlock {
    fn write_header(&mut self, output: &OutputStorage) {
        let h = &mut self.packer.header;

        h.targets = output.targets;
        h.t_cpu = output.t_cpu;
        h.v_bat = output.vbat;

        defmt::debug!(
            "{}",
            crate::main_data_storage::header_printer::HeaderPrinter(h)
        );
    }

    fn push_data(&mut self, result: Option<u32>, channel: FChannel) -> bool {
        defmt::trace!("DataPage::push_data(result={}, ch={})", result, channel);
        let v = if let Some(r) = result {
            let diff = r as i32
                - unsafe { core::mem::transmute::<u32, i32>(self.prevs[channel as usize]) };
            self.prevs[channel as usize] = r;
            diff
        } else {
            0
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
    pub fn new(fref_mul: f32, crc_calc: Arc<Mutex<stm32l4xx_hal::crc::Crc>>) -> Self {
        let mut master_counter_info = MasterCounter::acquire();
        master_counter_info.want_start();

        Self {
            master_counter_info,
            next_page_number: 0,
            crc_calc,
            fref_mul: fref_mul,
            page_aqured: false,
        }
    }
}

impl WriteController<DataBlock> for CpuFlashDiffWriter {
    fn try_create_new_page(&mut self) -> Result<DataBlock, FreeRtosError> {
        if !self.page_aqured {
            if let Some(ep) = crate::main_data_storage::find_next_empty_page(self.next_page_number)
            {
                defmt::info!("Aquaering page {}", ep);
                self.next_page_number = ep;
            } else {
                defmt::error!("Aquaering page failed, memory full!");
                return Err(freertos_rust::FreeRtosError::OutOfMemory);
            }

            let (base_interval_ms, interleave_ratio, fref) =
                match settings::settings_action::<_, _, _, ()>(Duration::ms(10), |(settings, _)| {
                    Ok((
                        settings.writeConfig.BaseInterval_ms,
                        [
                            settings.writeConfig.PWriteDevider,
                            settings.writeConfig.TWriteDevider,
                        ],
                        settings.Fref,
                    ))
                }) {
                    Ok(r) => r,
                    Err(settings::SettingActionError::AccessError(e)) => return Err(e),
                    _ => unreachable!(),
                };

            let packer = DataBlockPacker::builder()
                .set_ids(
                    self.next_page_number.checked_sub(1).unwrap_or_default(),
                    self.next_page_number,
                )
                .set_size(crate::main_data_storage::flash_page_size() as usize)
                .set_timestamp(self.master_counter_info.uptime_ms())
                .set_fref(self.fref_mul * fref as f32)
                .set_write_cfg(base_interval_ms, interleave_ratio)
                .build();

            let res = DataBlock {
                packer,
                dest_page: self.next_page_number,
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
                    !crc_guard.result() // результат инвертируется, чтобы соотвектсвовать zlib
                })
                .unwrap_or_default()
        }) {
            if let Ok(mut page_accessor) = crate::main_data_storage::select_page(page.dest_page) {
                let len = data.len();
                if let Ok(()) = page_accessor.write(data) {
                    defmt::info!(
                        "Write page {}, {} values ({} bytes) -> {}",
                        id,
                        input_count,
                        input_count * core::mem::size_of::<u32>(),
                        len
                    );
                    return write_controller::PageWriteResult::Succes(id);
                }
            }

            defmt::error!("Failed to get page {}!", page.dest_page);
            return write_controller::PageWriteResult::Fail(id);
        } else {
            defmt::error!("Page {} generation failed!", id);
            write_controller::PageWriteResult::Fail(id)
        }
    }
}
