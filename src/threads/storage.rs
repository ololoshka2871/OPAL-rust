use alloc::vec::Vec;

use emfat_rust::{emfat_entry, emfat_t};

use usbd_scsi::{BlockDevice, BlockDeviceError};

pub struct EMfatStorage {
    ctx: emfat_t,
    fstable: Vec<emfat_entry>,
}

/*
const char *autorun_file =
    "[autorun]\r\n"
    "label=emfat test drive\r\n"
    "ICON=icon.ico\r\n";

const char *readme_file =
    "This is readme file\r\n";
*/

/*
    // name          dir    lvl offset  size             max_size        user  read               write
    { "",            true,  0,  0,      0,               0,              0,    NULL,              NULL }, // root
    { "autorun.inf", false, 1,  0,      AUTORUN_SIZE,    1*1024*1024*1024,   0,    autorun_read_proc, NULL }, // autorun.inf
    { "icon.ico",    false, 1,  0,      ICON_SIZE,       1*1024*1024*1024,      0,    icon_read_proc,    NULL }, // icon.ico
    { "drivers",     true,  1,  0,      0,               0,              0,    NULL,              NULL }, // drivers/
    { "readme.txt",  false, 2,  0,      README_SIZE,     1*1024*1024*1024,    0,    readme_read_proc,  NULL }, // drivers/readme.txt
    { "abc.txt",  false, 2,  0,      README_SIZE,     1*1024*1024*1024-32768+8192,    0,    readme_read_proc,  NULL }, // drivers/readme.txt
    { NULL }
*/

struct StaticData {
    data: &'static str,
}

static AUTORUN: &str = r#"[autorun]
label=emfat test drive
ICON=icon.ico
"#;

static AUTORUN_INFO: StaticData = StaticData { data: AUTORUN };

static README: &str = "This is readme file\r\n";

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

    core::ptr::copy_nonoverlapping(dptr.data.as_ptr(), dest, to_read);
}

impl EMfatStorage {
    pub fn new(label: &str) -> EMfatStorage {
        let mut res = EMfatStorage {
            ctx: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
            fstable: EMfatStorage::build_files_table(),
        };

        unsafe { emfat_rust::emfat_init(&mut res.ctx, label.as_ptr(), res.fstable.as_mut_ptr()) };

        res
    }

    fn build_files_table() -> Vec<emfat_entry> {
        let mut res = Vec::<emfat_entry>::new();

        // /
        res.push(
            emfat_rust::EntryBuilder::new()
                .name("")
                .dir(true)
                .lvl(0)
                .offset(0)
                .size(0)
                .max_size(0)
                .build(),
        );

        // /autorun.inf
        let ptr = &AUTORUN_INFO as *const StaticData;
        res.push(
            emfat_rust::EntryBuilder::new()
                .name("autorun.inf")
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(AUTORUN.len())
                .max_size(AUTORUN.len())
                .read_cb(const_reader)
                .user_data(ptr as usize)
                .build(),
        );

        // /readme.inf
        let ptr = &README_INFO as *const StaticData;
        res.push(
            emfat_rust::EntryBuilder::new()
                .name("readme.txt")
                .dir(false)
                .lvl(1)
                .offset(0)
                .size(README.len())
                .max_size(README.len())
                .read_cb(const_reader)
                .user_data(ptr as usize)
                .build(),
        );

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

    fn write_block(&mut self, lba: u32, block: &[u8]) -> Result<(), BlockDeviceError> {
        unsafe {
            emfat_rust::emfat_write(&mut self.ctx, block.as_ptr(), lba, 1);
        }
        Ok(())
    }

    fn max_lba(&self) -> u32 {
        self.ctx.disk_sectors // Это не размер а максимальный номер блока по 512 байт
    }
}
