#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("./bindings/emfat-bindings.rs");

// emfat требует strlen(), стандартный чего-тоне линкуется, напишем свой.
#[no_mangle]
pub unsafe extern "C" fn strlen(cs: *const c_char) -> size_t {
    let mut res: size_t = 0;
    while *cs.add(res) != '\0' as u8 {
        res += 1;
    }
    res
}

pub struct EntryBuilder {
    entry: emfat_entry,
}

impl EntryBuilder {
    pub fn new() -> EntryBuilder {
        EntryBuilder {
            entry: emfat_entry {
                name: "".as_ptr(),
                dir: false,
                level: 0,
                offset: 0,
                curr_size: 0,
                max_size: 0,
                user_data: 0,

                cma_time: [0, 0, 0],
                readcb: None,
                writecb: None,
                priv_: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
            },
        }
    }

    pub fn terminator_entry() -> emfat_entry {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    pub fn name(mut self, name: &'static str) -> Self {
        self.entry.name = assert_null_terminated(name).as_ptr();
        self
    }

    pub fn dir(mut self, is_dir: bool) -> Self {
        self.entry.dir = is_dir;
        self
    }

    pub fn lvl(mut self, lvl: i32) -> Self {
        self.entry.level = lvl;
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.entry.offset = offset as u32;
        self
    }

    pub fn size(mut self, size: usize) -> Self {
        self.entry.curr_size = size as u32;
        self
    }

    pub fn max_size(mut self, max_size: usize) -> Self {
        self.entry.max_size = max_size as u32;
        self
    }

    pub fn user_data(mut self, userdata: usize) -> Self {
        self.entry.user_data = userdata;
        self
    }

    //-------------

    pub fn read_cb(
        mut self,
        cb: unsafe extern "C" fn(dest: *mut u8, size: i32, offset: u32, userdata: usize),
    ) -> Self {
        self.entry.readcb = Some(cb);
        self
    }

    pub fn write_cb(
        self,
        _cb: unsafe extern "C" fn(data: *const u8, size: i32, offset: u32, userdata: usize),
    ) -> Self {
        todo!("Not realised in library!");
        //self.entry.writecb = Some(cb);
        //self
    }

    //-------------

    pub fn build(self) -> emfat_entry {
        // files mast provide at leas 1 callback
        assert!(!(!self.entry.dir && self.entry.readcb == None && self.entry.writecb == None));
        self.entry
    }
}

fn assert_null_terminated(string: &str) -> &str {
    assert_eq!(string.chars().rev().next().unwrap(), '\0');
    string
}

pub fn emfat_rust_init(
    emfat: &mut emfat_t,
    disk_label: &str,
    entries: *mut emfat_entry_t,
) {
    unsafe {
        assert!(emfat_init(emfat, assert_null_terminated(disk_label).as_ptr(), entries));
    };
}