use core::usize;

use super::PageAccessor;

pub const INTERNAL_FLASH_PAGE_SIZE: usize = 2048;

#[link_section = ".writer_test_area.place"]
static STORAGE: [u8; INTERNAL_FLASH_PAGE_SIZE * 2] = [0u8; INTERNAL_FLASH_PAGE_SIZE * 2];

pub struct InternalPageAccessor {
    ptr: *mut u8,
}

impl PageAccessor for InternalPageAccessor {
    fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), ()> {
        Err(())
    }

    fn read_to(&self, offset: usize, dest: &mut [u8]) {
        unsafe {
            core::ptr::copy_nonoverlapping(self.ptr.add(offset), dest.as_mut_ptr(), dest.len())
        }
    }
}

pub fn select_page(page: u32) -> Result<impl PageAccessor, ()> {
    assert!(page < 2);
    Ok(InternalPageAccessor {
        ptr: unsafe { (STORAGE.as_ptr() as *mut u8).add(page as usize * INTERNAL_FLASH_PAGE_SIZE) },
    })
}

pub fn flash_erease() -> Result<(), ()> {
    Err(())
}

pub fn flash_size() -> usize {
    STORAGE.len()
}
