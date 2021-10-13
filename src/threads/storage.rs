use alloc::vec::Vec;

use emfat_rust::{emfat_entry, emfat_t};

use usbd_scsi::{BlockDevice, BlockDeviceError};

pub struct EMfatStorage {
    ctx: emfat_t,
    fstable: Vec<emfat_entry>,
}

struct StaticData {
    data: &'static str,
}

// terminate strings with '\0' for strlen() compatible

static README: &str = "# СКТБ \"ЭЛПА\": Автономный регистратор давления\n\
\n\
Этот виртуальный диск предоставляет доступ к содержимому внутреннего накопителя устройства.\n\
\n\
- Для расшифровки содержимого используйте программу %TODO%.\n\
- Коэффициенты полиномов для рассчета находятся в файле %TODO%\n\
- Для управление функционалом устройства используйте программу KalibratorGUI\n";

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

unsafe extern "C" fn null_read(_dest: *mut u8, _size: i32, _offset: u32, _userdata: usize) {}

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
        // TODO incapsulate constructing files
        let mut res = Vec::<emfat_entry>::new();

        // /
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

        // /readme.inf
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

        // /null
        res.push(
            emfat_rust::EntryBuilder::new()
                .name("Testfile.bin\0")
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(1024 * 1024 * 10)
                .max_size(1024 * 1024 * 20)
                .read_cb(null_read)
                .build(),
        );

        res.push(emfat_rust::EntryBuilder::terminator_entry());

        res
    }
}

impl BlockDevice for EMfatStorage {
    const BLOCK_BYTES: usize = 512;

    fn read_block(&self, lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        unsafe {
            // костыль, либа дает константную ссылку, принудительно конвернируем
            // в неконстантный указатель
            let ctx = &self.ctx as *const emfat_t as *mut emfat_t;
            emfat_rust::emfat_read(ctx, block.as_mut_ptr(), lba, 1);
        }
        Ok(())
    }

    fn write_block(&mut self, _lba: u32, _block: &[u8]) -> Result<(), BlockDeviceError> {
        Err(BlockDeviceError::WriteError)
    }

    fn max_lba(&self) -> u32 {
        self.ctx.disk_sectors // Это не размер а максимальный номер блока по 512 байт
    }
}
