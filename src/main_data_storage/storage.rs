use alloc::boxed::Box;

use stm32l4xx_hal::traits::flash;

use super::PageAccessor;

pub trait Storage: {
    fn select_page(&mut self, page: u32) -> Result<Box<dyn PageAccessor>, ()>;
    fn flash_page_size(&mut self) -> u32;
    fn flash_size(&mut self) -> usize;
    fn flash_size_pages(&mut self) -> u32;
    fn flash_erease(&mut self) -> Result<(), flash::Error>;
}
