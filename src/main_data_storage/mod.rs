pub mod data_page;
pub mod write_controller;

pub mod test_writer;
pub mod cpu_flash_diff_writer;

pub fn flash_erease() -> Result<(), ()> {
    Err(())
}

pub const fn flash_page_size() -> u32 {
    // MT25QU01GBBB8E12 Subsector = 4KB
    4096
}
