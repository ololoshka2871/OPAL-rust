use flash_settings_rs::StoragePolicy;
use stm32l4xx_hal::flash::{self, Read, WriteErase};

/// https://docs.rs/stm32l4xx-hal/0.6.0/stm32l4xx_hal/flash/index.html
pub struct FlasRWPolcy<'a> {
    flash: &'a mut stm32l4xx_hal::flash::Parts,
    page: flash::FlashPage,
}

impl<'a> FlasRWPolcy<'a> {
    pub fn new<T>(flash: &'a mut stm32l4xx_hal::flash::Parts, data: &T) -> Self {
        const PAGE0_ADDR: usize = flash::FlashPage(0).to_address();
        const PAGE_SIZE: usize = flash::FlashPage(1).to_address() - PAGE0_ADDR;

        let addres = data as *const T as usize;
        assert!(addres > PAGE0_ADDR);
        assert_eq!(addres % PAGE_SIZE, 0);

        Self {
            flash,
            page: flash::FlashPage(addres),
        }
    }
}

impl<'a> StoragePolicy<stm32l4xx_hal::traits::flash::Error> for FlasRWPolcy<'a> {
    unsafe fn store(&self, data: &[u8]) -> Result<(), stm32l4xx_hal::traits::flash::Error> {
        if data.len() % ::core::mem::size_of::<u64>() != 0 {
            return Err(stm32l4xx_hal::traits::flash::Error::Illegal);
        }
        // guard. Autolock after drop()
        let mut prog = self.flash.keyr.unlock_flash(&mut self.flash.sr, &mut self.flash.cr)?;

        prog.erase_page(self.page)?;
        prog.write_native(self.page.to_address(), ::core::slice::from_raw_parts(
            data.as_ptr() as *const u64, data.len() / ::core::mem::size_of::<u64>()))?;

        Ok(())
    }

    unsafe fn load(&self, data: &mut [u8]) -> Result<(), stm32l4xx_hal::traits::flash::Error> {
        let mut prog = self.flash.keyr.unlock_flash(&mut self.flash.sr, &mut self.flash.cr)?;

        prog.read_native(self.page.to_address(), data);

        Ok(())
    }
}
