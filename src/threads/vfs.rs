mod callbacks;
mod static_data;

use core::usize;

use alloc::vec::Vec;

use emfat_rust::{emfat_entry, emfat_t, EntryBuilder};

use heatshrink_rust::CompressedData;
use my_proc_macro::c_str;
use usbd_scsi::{BlockDevice, BlockDeviceError};

pub struct EMfatStorage {
    ctx: emfat_t,
    fstable: Vec<emfat_entry>,
}

struct StaticBinData {
    data: &'static [u8],
}

// terminate strings with '\0' c_str("text") for strlen() compatible

impl EMfatStorage {
    pub fn new(disk_label: &str) -> EMfatStorage {
        let mut res = EMfatStorage {
            ctx: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
            fstable: EMfatStorage::build_files_table(),
        };
        emfat_rust::emfat_rust_init(&mut res.ctx, disk_label, res.fstable.as_mut_ptr());
        res
    }

    fn build_files_table() -> Vec<emfat_entry> {
        use callbacks::{flash_read, meminfo_read, settings_read, unpack_reader};
        use static_data::{DRIVER_INF_COMPRESSED, PROTO_COMPRESSED, README_COMPRESSED};

        defmt::trace!("EmFat: Registring virtual files:");

        let mut res: Vec<emfat_entry> = Vec::new();

        defmt::trace!("EmFat: .. /");
        res.push(
            EntryBuilder::new()
                .name(c_str!(""))
                .dir(true)
                .lvl(0)
                .offset(0)
                .size(0)
                .max_size(0)
                .build(),
        );

        defmt::trace!("EmFat: .. /Readme.txt");
        res.push(
            EntryBuilder::new()
                .name(c_str!("Readme.txt"))
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(README_COMPRESSED.original_size)
                .max_size(README_COMPRESSED.original_size)
                .read_cb(Some(unpack_reader))
                .user_data(&README_COMPRESSED as *const CompressedData as usize)
                .build(),
        );

        defmt::trace!("EmFat: .. /driver.inf");
        res.push(
            EntryBuilder::new()
                .name(c_str!("driver.inf"))
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(DRIVER_INF_COMPRESSED.original_size)
                .max_size(DRIVER_INF_COMPRESSED.original_size)
                .read_cb(Some(unpack_reader))
                .user_data(&DRIVER_INF_COMPRESSED as *const CompressedData as usize)
                .build(),
        );

        defmt::trace!("EmFat: .. /proto.prt");
        res.push(
            EntryBuilder::new()
                .name(c_str!("proto.prt"))
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(PROTO_COMPRESSED.original_size)
                .max_size(PROTO_COMPRESSED.original_size)
                .read_cb(Some(unpack_reader))
                .user_data(&PROTO_COMPRESSED as *const CompressedData as usize)
                .build(),
        );

        defmt::trace!("EmFat: .. /settings.var");
        res.push(
            EntryBuilder::new()
                .name(c_str!("config.var"))
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(2048) // noauto, размер может меняться - это генерированный текст
                .max_size(2048)
                .read_cb(Some(settings_read))
                .build(),
        );

        defmt::trace!("EmFat: .. /storage.var");
        res.push(
            EntryBuilder::new()
                .name(c_str!("storage.var"))
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(512) // noauto, размер может меняться - это генерированный текст
                .max_size(2048)
                .read_cb(Some(meminfo_read))
                .build(),
        );

        defmt::trace!("EmFat: .. /data.hs");
        let flash_size = crate::main_data_storage::flash_size();
        res.push(
            EntryBuilder::new()
                .name(c_str!("data.hs"))
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(flash_size)
                .max_size(flash_size)
                .read_cb(Some(flash_read))
                .build(),
        );

        /*
        let mut master = alloc::boxed::Box::new(
            crate::sensors::freqmeter::master_counter::MasterCounter::acquire(),
        );
        master.want_start();

        defmt::trace!("EmFat: .. /master.val");
        res.push(
            EntryBuilder::new()
                .name(c_str!("master.val"))
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(10)
                .max_size(10)
                .read_cb(master_read)
                .user_data(alloc::boxed::Box::into_raw(master) as usize)
                .build(),
        );
        */

        /*
        defmt::trace!("EmFat: .. /Testfile.bin");
        res.push(
            EntryBuilder::new()
                .name(c_str!("Testfile.bin"))
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(1024 * 10)
                .max_size(1024 * 20)
                .read_cb(null_read)
                .build(),
        );
        */

        /*
        defmt::trace!("EmFat: .. /fill.x");
        res.push(
            EntryBuilder::new()
                .name(c_str!("fill.x"))
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(65600 * 8 * 512)
                .max_size(65600 * 8 * 512)
                .read_cb(null_read)
                .build(),
        );*/

        res.push(EntryBuilder::terminator_entry());

        res
    }
}

impl BlockDevice for EMfatStorage {
    const BLOCK_BYTES: usize = 512;

    fn read_block(&mut self, lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        //defmt::debug!("SCSI: Read LBA block {}", lba);
        unsafe {
            emfat_rust::emfat_read(&mut self.ctx, block.as_mut_ptr(), lba, 1);
        }
        Ok(())
    }

    fn write_block(&mut self, _lba: u32, _block: &[u8]) -> Result<(), BlockDeviceError> {
        //defmt::trace!("SCSI: Write LBA block {}", lba);
        //unsafe { emfat_rust::emfat_write(&mut self.ctx, block.as_ptr(), lba, 1) }
        //Ok(())
        Err(BlockDeviceError::HardwareError)
    }

    fn max_lba(&self) -> u32 {
        //defmt::trace!("SCSI: Get max LBA {}", self.ctx.disk_sectors);
        self.ctx.disk_sectors // Это не размер а максимальный номер блока по 512 байт
    }

    fn is_write_protected(&self) -> bool {
        true
    }
}
