pub type size_t = usize;
pub type c_int = i32;
pub type c_char = u8;

pub type emfat_readcb_t = Option<
    unsafe extern "C" fn(dest: *mut u8, size: c_int, offset: u32, userdata: size_t),
>;

pub type emfat_writecb_t = Option<
    unsafe extern "C" fn(
        data: *const u8,
        size: c_int,
        offset: u32,
        userdata: size_t,
    ),
>;

pub type emfat_entry_t = emfat_entry;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct emfat_entry {
    pub name: *const c_char,
    pub dir: bool,
    pub level: c_int,
    pub offset: u32,
    pub curr_size: u32,
    pub max_size: u32,
    pub user_data: size_t,
    #[doc = "< create/mod/access time in unix format"]
    pub cma_time: [u32; 3usize],
    pub readcb: emfat_readcb_t,
    pub writecb: emfat_writecb_t,
    pub priv_: emfat_entry__bindgen_ty_1,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct emfat_entry__bindgen_ty_1 {
    pub first_clust: u32,
    pub last_clust: u32,
    pub last_reserved: u32,
    pub num_subentry: u32,
    pub top: *mut emfat_entry_t,
    pub sub: *mut emfat_entry_t,
    pub next: *mut emfat_entry_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct emfat_t {
    pub vol_size: u64,
    pub disk_sectors: u32,
    pub vol_label: *const c_char,
    pub priv_: emfat_t__bindgen_ty_1,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct emfat_t__bindgen_ty_1 {
    pub boot_lba: u32,
    pub fsinfo_lba: u32,
    pub fat1_lba: u32,
    pub fat2_lba: u32,
    pub root_lba: u32,
    pub num_clust: u32,
    pub entries: *mut emfat_entry_t,
    pub last_entry: *mut emfat_entry_t,
    pub num_entries: c_int,
}

extern "C" {
    pub fn emfat_init(
        emfat: *mut emfat_t,
        label: *const c_char,
        entries: *mut emfat_entry_t,
    ) -> bool;

    pub fn emfat_read(
        emfat: *mut emfat_t,
        data: *mut u8,
        sector: u32,
        num_sectors: c_int,
    );

    pub fn emfat_write(
        emfat: *mut emfat_t,
        data: *const u8,
        sector: u32,
        num_sectors: c_int,
    );

    pub fn emfat_cma_time_from_unix(unix_time: u32) -> u32;
}
