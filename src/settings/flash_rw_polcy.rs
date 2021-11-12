use core::ops::DerefMut;

use alloc::sync::Arc;
use flash_settings_rs::StoragePolicy;
use freertos_rust::{Duration, Mutex};
use stm32l4xx_hal::flash::{self, Read, WriteErase};

pub struct Placeholder<T> {
    _body: T,
    _crc: u64,
}

/// https://docs.rs/stm32l4xx-hal/0.6.0/stm32l4xx_hal/flash/index.html
pub struct FlasRWPolcy {
    flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>,
    page: flash::FlashPage,
}

impl FlasRWPolcy {
    pub fn create<T>(
        data: &Placeholder<T>,
        flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>,
    ) -> Self {
        const PAGE0_ADDR: usize = flash::FlashPage(0).to_address();
        const PAGE_SIZE: usize = flash::FlashPage(1).to_address() - PAGE0_ADDR;

        let addres = data as *const Placeholder<T> as usize;
        assert!(addres > PAGE0_ADDR);
        assert_eq!(addres % PAGE_SIZE, 0);

        Self {
            flash,
            page: flash::FlashPage((addres - PAGE0_ADDR) / PAGE_SIZE),
        }
    }

    fn len_in_u64_aligned(data: &[u8]) -> usize {
        if data.len() % ::core::mem::size_of::<u64>() != 0 {
            data.len() % ::core::mem::size_of::<u64>() + 1
        } else {
            data.len() % ::core::mem::size_of::<u64>()
        }
    }
}

impl StoragePolicy<flash::Error> for FlasRWPolcy {
    unsafe fn store(&mut self, data: &[u8]) -> Result<(), flash::Error> {
        if let Ok(mut flash_guard) = self.flash.lock(Duration::infinite()) {
            let flash = flash_guard.deref_mut();
            let mut prog = flash.keyr.unlock_flash(&mut flash.sr, &mut flash.cr)?;

            let len_in_u64_aligned = Self::len_in_u64_aligned(data);

            prog.erase_page(self.page)?;
            prog.write_native(
                self.page.to_address(),
                ::core::slice::from_raw_parts(data.as_ptr() as *const u64, len_in_u64_aligned),
            )?;

            let crc = [123456789u64];
            prog.write_native(self.page.to_address(), &crc)?;

            Ok(())
        } else {
            panic!()
        }
    }

    unsafe fn load(
        &mut self,
        data: &mut [u8],
    ) -> Result<(), flash_settings_rs::LoadError<flash::Error>> {
        if let Ok(mut flash_guard) = self.flash.lock(Duration::infinite()) {
            let flash = flash_guard.deref_mut();
            let prog = flash
                .keyr
                .unlock_flash(&mut flash.sr, &mut flash.cr)
                .map_err(|e| flash_settings_rs::LoadError::ReadError(e))?;
            prog.read_native(self.page.to_address(), data);

            let len_aligned = Self::len_in_u64_aligned(data) * ::core::mem::size_of::<u64>();

            let mut crc = 0_u64;
            prog.read_native(
                self.page.to_address() + len_aligned,
                ::core::slice::from_raw_parts_mut(
                    &mut crc as *mut u64 as *mut u8,
                    ::core::mem::size_of::<u64>(),
                ),
            );

            return if crc != 0x123456789u64 {
                Err(flash_settings_rs::LoadError::ConststenceError)
            } else {
                Ok(())
            };
        } else {
            panic!()
        }
    }
}
