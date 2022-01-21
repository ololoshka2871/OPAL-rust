use core::{ops::DerefMut, usize};

use alloc::{sync::Arc, vec::Vec};
use freertos_rust::{Duration, Mutex, MutexGuard, MutexNormal};
use stm32l4xx_hal::{flash::WriteErase, traits::flash};

use super::PageAccessor;

pub const INTERNAL_FLASH_PAGE_SIZE: usize = 2048;
pub const INTERNAL_FLASH_PAGES: usize = 8;

#[link_section = ".writer_test_area.place"]
static STORAGE: [u8; INTERNAL_FLASH_PAGE_SIZE * INTERNAL_FLASH_PAGES] =
    [0u8; INTERNAL_FLASH_PAGE_SIZE * INTERNAL_FLASH_PAGES];

static mut FLASH: Option<Arc<Mutex<stm32l4xx_hal::flash::Parts>>> = None;

pub struct InternalPageAccessor<'a> {
    _guard: MutexGuard<'a, stm32l4xx_hal::flash::Parts, MutexNormal>,
    ptr: *mut u8,
}

impl<'a> PageAccessor for InternalPageAccessor<'a> {
    fn write(&mut self, data: Vec<u8>) -> Result<(), flash::Error> {
        let flash = self._guard.deref_mut();
        let mut prog = flash.keyr.unlock_flash(&mut flash.sr, &mut flash.cr)?;

        let len_in_u64_aligned =
            crate::support::len_in_u64_aligned::len_in_u64_aligned(data.as_slice());

        prog.write_native(self.ptr as usize, unsafe {
            ::core::slice::from_raw_parts(data.as_ptr() as *const u64, len_in_u64_aligned)
        })
    }

    fn read_to(&self, offset: usize, dest: &mut [u8]) {
        unsafe {
            core::ptr::copy_nonoverlapping(self.ptr.add(offset), dest.as_mut_ptr(), dest.len())
        }
    }
}

pub fn select_page(page: u32) -> Result<impl PageAccessor, ()> {
    assert!(page < flash_size_pages());
    if let Some(flash) = unsafe { &FLASH } {
        if let Ok(guard) = flash.lock(Duration::infinite()) {
            return Ok(InternalPageAccessor {
                _guard: guard,
                ptr: unsafe {
                    (STORAGE.as_ptr() as *mut u8).add(page as usize * INTERNAL_FLASH_PAGE_SIZE)
                },
            });
        }
    }
    Err(())
}

pub fn flash_erease() -> Result<(), ()> {
    Err(())
}

pub fn find_next_empty_page(start: u32) -> Option<u32> {
    if start < flash_size_pages() {
        Some(start)
    } else {
        None
    }
}

pub fn flash_size() -> usize {
    STORAGE.len()
}

pub fn flash_size_pages() -> u32 {
    (STORAGE.len() / INTERNAL_FLASH_PAGE_SIZE) as u32
}

pub fn init(flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>) {
    unsafe { FLASH = Some(flash) };
}
