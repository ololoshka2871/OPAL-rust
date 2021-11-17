use alloc::vec::Vec;

use emfat_rust::{emfat_entry, emfat_t};

use freertos_rust::Duration;
use usbd_scsi::{BlockDevice, BlockDeviceError};

pub struct EMfatStorage {
    ctx: emfat_t,
    fstable: Vec<emfat_entry>,
}

struct StaticData {
    data: &'static str,
}

// terminate strings with '\0' for strlen() compatible

static README: &str = "# СКТБ \"ЭЛПА\": Автономный регистратор давления\r\n\
\r\n\
Этот виртуальный диск предоставляет доступ к содержимому внутреннего накопителя устройства.\n\
\r\n\
- Для расшифровки содержимого используйте программу %TODO%.\r\n\
- Коэффициенты полиномов для рассчета находятся в файле config.cfg (формат json)\r\n\
- Для управление функционалом устройства используйте программу KalibratorGUI\r\n";

static README_INFO: StaticData = StaticData { data: README };

unsafe extern "C" fn const_reader(dest: *mut u8, size: i32, offset: u32, userdata: usize) {
    let dptr = &*(userdata as *const StaticData);
    if offset as usize > dptr.data.len() {
        return;
    }
    let to_read = if offset as usize + size as usize > dptr.data.len() {
        dptr.data.len() - offset as usize
    } else {
        size as usize
    };

    core::ptr::copy_nonoverlapping(dptr.data.as_ptr().add(offset as usize), dest, to_read);
}

//unsafe extern "C" fn null_read(_dest: *mut u8, _size: i32, _offset: u32, _userdata: usize) {}

unsafe extern "C" fn settings_read(dest: *mut u8, size: i32, _offset: u32, _userdata: usize) {
    match crate::settings::settings_action(Duration::ms(2), |s| serde_json::to_string_pretty(&s)) {
        Ok(s) => {
            let src = s.as_bytes();
            let offset = _offset as usize;
            if src.len() > offset {
                let src = &src[offset..];
                let to_write = core::cmp::min(size as usize, src.len());
                core::ptr::copy_nonoverlapping(src.as_ptr(), dest, to_write);

                // забиваем буфер пробелами до конца, чтобы в блокноте он нормально выглядел
                core::ptr::write_bytes(dest.add(src.len()), b' ', size as usize - to_write);
            }
        }
        Err(crate::settings::SettingActionError::AccessError(e)) => {
            defmt::error!("Failed to setialise settings: {}", defmt::Debug2Format(&e));
        }
        Err(crate::settings::SettingActionError::ActionError(e)) => {
            defmt::error!(
                "Failed to setialise settings: {}",
                defmt::Display2Format(&e)
            );
        }
    }
}

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
        defmt::trace!("EmFat: Registring virtual files:");

        // TODO incapsulate constructing files
        let mut res = Vec::<emfat_entry>::new();

        defmt::trace!("EmFat: .. /");
        res.push(
            emfat_rust::EntryBuilder::new()
                .name("\0")
                .dir(true)
                .lvl(0)
                .offset(0)
                .size(0)
                .max_size(0)
                .build(),
        );

        defmt::trace!("EmFat: .. /Readme.txt");
        let ptr = &README_INFO as *const StaticData;
        res.push(
            emfat_rust::EntryBuilder::new()
                .name("Readme.txt\0")
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(README.len())
                .max_size(README.len())
                .read_cb(const_reader)
                .user_data(ptr as usize)
                .build(),
        );

        defmt::trace!("EmFat: .. /settings.json");
        res.push(
            emfat_rust::EntryBuilder::new()
                .name("config.cfg\0")
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(1024) // noauto, размер может меняться - это генерированный текст
                .max_size(1024)
                .read_cb(settings_read)
                .build(),
        );

        /*
        defmt::trace!("EmFat: .. /Testfile.bin");
        res.push(
            emfat_rust::EntryBuilder::new()
                .name("Testfile.bin\0")
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
            emfat_rust::EntryBuilder::new()
                .name("fill.x\0")
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(65600 * 8 * 512)
                .max_size(65600 * 8 * 512)
                .read_cb(null_read)
                .build(),
        );*/

        res.push(emfat_rust::EntryBuilder::terminator_entry());

        res
    }
}

impl BlockDevice for EMfatStorage {
    const BLOCK_BYTES: usize = 512;

    fn read_block(&mut self, lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        defmt::trace!("SCSI: Read LBA block {}", lba);
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
        defmt::trace!("SCSI: Get max LBA {}", self.ctx.disk_sectors);
        self.ctx.disk_sectors // Это не размер а максимальный номер блока по 512 байт
    }

    fn is_write_protected(&self) -> bool {
        true
    }
}
