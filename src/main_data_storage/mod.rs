pub mod data_page;
pub mod write_controller;

pub mod cpu_flash_diff_writer;
pub mod test_writer;

mod internal_storage;

pub trait PageAccessor {
    fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), ()>;
    fn read_to(&self, offset: usize, dest: &mut [u8]);
}

pub fn flash_erease() -> Result<(), ()> {
    internal_storage::flash_erease()
}

pub fn select_page(page: u32) -> Result<impl PageAccessor, ()> {
    internal_storage::select_page(page)
}

pub fn flash_page_size() -> u32 {
    // MT25QU01GBBB8E12 Subsector = 4KB
    //4096

    // CPU own flash
    internal_storage::INTERNAL_FLASH_PAGE_SIZE as u32
}

pub fn flash_size() -> usize {
    internal_storage::flash_size()
}
