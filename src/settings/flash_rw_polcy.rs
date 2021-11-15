use core::ops::DerefMut;

use alloc::sync::Arc;
use flash_settings_rs::StoragePolicy;
use freertos_rust::{Duration, Mutex};
use stm32l4xx_hal::flash::{self, WriteErase};

pub struct Placeholder<T> {
    _body: T,
    _crc: u64,
}

/// https://docs.rs/stm32l4xx-hal/0.6.0/stm32l4xx_hal/flash/index.html
pub struct FlasRWPolcy {
    flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>,
    crc: Arc<Mutex<stm32l4xx_hal::crc::Crc>>,
    page: flash::FlashPage,
}

impl FlasRWPolcy {
    pub fn create<T>(
        data: &Placeholder<T>,
        flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>,
        crc: Arc<Mutex<stm32l4xx_hal::crc::Crc>>,
    ) -> Self {
        const PAGE0_ADDR: usize = flash::FlashPage(0).to_address();
        const PAGE_SIZE: usize = flash::FlashPage(1).to_address() - PAGE0_ADDR;

        let addres = data as *const Placeholder<T> as usize;
        assert!(addres > PAGE0_ADDR);
        assert_eq!(addres % PAGE_SIZE, 0);

        Self {
            flash,
            crc,
            page: flash::FlashPage((addres - PAGE0_ADDR) / PAGE_SIZE),
        }
    }

    fn len_in_u64_aligned(data: &[u8]) -> usize {
        if data.len() % ::core::mem::size_of::<u64>() != 0 {
            data.len() / ::core::mem::size_of::<u64>() + 1
        } else {
            data.len() / ::core::mem::size_of::<u64>()
        }
    }

    // https://docs.rs/stm32l4xx-hal/0.6.0/stm32l4xx_hal/crc/index.html
    fn crc(&mut self, data: &[u8]) -> u32 {
        if let Ok(mut crc_guard) = self.crc.lock(Duration::infinite()) {
            crc_guard.reset();
            crc_guard.feed(data);
            crc_guard.result()
        } else {
            panic!("Failed to lock crc module");
        }
    }
}

impl StoragePolicy<flash::Error> for FlasRWPolcy {
    unsafe fn store(&mut self, data: &[u8]) -> Result<(), flash::Error> {
        let current_crc = [self.crc(data) as u64];

        if let Ok(mut flash_guard) = self.flash.lock(Duration::infinite()) {
            let flash = flash_guard.deref_mut();
            let mut prog = flash.keyr.unlock_flash(&mut flash.sr, &mut flash.cr)?;

            let len_in_u64_aligned = Self::len_in_u64_aligned(data);

            prog.erase_page(self.page)?;
            prog.write_native(
                self.page.to_address(),
                ::core::slice::from_raw_parts(data.as_ptr() as *const u64, len_in_u64_aligned),
            )?;

            prog.write_native(
                self.page.to_address() + len_in_u64_aligned * ::core::mem::size_of::<u64>(),
                &current_crc,
            )?;

            Ok(())
        } else {
            panic!("Failed to lock flash")
        }
    }

    unsafe fn load(
        &mut self,
        data: &mut [u8],
    ) -> Result<(), flash_settings_rs::LoadError<flash::Error>> {
        core::ptr::copy_nonoverlapping(
            self.page.to_address() as *const _,
            data.as_mut_ptr(),
            data.len(),
        );

        let len_aligned = Self::len_in_u64_aligned(data) * ::core::mem::size_of::<u64>();
        let mut crc: u64 = core::mem::MaybeUninit::zeroed().assume_init();

        core::ptr::copy_nonoverlapping(
            (self.page.to_address() + len_aligned) as *const _,
            &mut crc,
            1,
        );

        if crc != self.crc(data) as u64 {
            Err(flash_settings_rs::LoadError::ConststenceError)
        } else {
            Ok(())
        }
    }
}
