use alloc::{sync::Arc, vec::Vec};
use freertos_rust::Mutex;

use stm32l4xx_hal::traits::flash;

pub mod data_page;
pub mod write_controller;

pub mod cpu_flash_diff_writer;
//pub mod test_writer;

pub(crate) mod header_printer;
mod internal_storage;

enum MemoryState {
    Undefined,
    PartialUsed(u32),
    FullUsed,
}

static mut NEXT_EMPTY_PAGE: MemoryState = MemoryState::Undefined;

pub trait PageAccessor {
    fn write(&mut self, data: Vec<u8>) -> Result<(), flash::Error>;
    fn read_to(&self, offset: usize, dest: &mut [u8]);
}

pub fn flash_erease() -> Result<(), ()> {
    internal_storage::flash_erease()
}

pub fn find_next_empty_page(start: u32) -> Option<u32> {
    match unsafe { &NEXT_EMPTY_PAGE } {
        MemoryState::Undefined => {}
        MemoryState::PartialUsed(next_free_page) => {
            if start <= *next_free_page {
                return Some(*next_free_page);
            }
        }
        MemoryState::FullUsed => return None,
    }

    if start < flash_size_pages() {
        for p in start..flash_size_pages() {
            let accessor = unsafe { select_page(p).unwrap_unchecked() };
            let mut header_blockchain: self_recorder_packet::DataPacketHeader =
                unsafe { core::mem::MaybeUninit::uninit().assume_init() };
            accessor.read_to(0, unsafe {
                core::slice::from_raw_parts_mut(
                    &mut header_blockchain as *mut _ as *mut u8,
                    core::mem::size_of_val(&header_blockchain),
                )
            });

            // признаком того, что флешка стерта является то, что там везде FF
            if core::cmp::min(
                header_blockchain.this_block_id,
                header_blockchain.prev_block_id,
            ) == u32::MAX
            {
                unsafe { NEXT_EMPTY_PAGE = MemoryState::PartialUsed(p) };
                return Some(p);
            }
        }
    }
    unsafe { NEXT_EMPTY_PAGE = MemoryState::FullUsed };
    None
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

pub fn flash_size_pages() -> u32 {
    internal_storage::flash_size_pages()
}

pub(crate) fn init(flash: Arc<Mutex<stm32l4xx_hal::flash::Parts>>) {
    internal_storage::init(flash);
    let next_free_page = find_next_empty_page(0);
    if let Some(next_free_page) = next_free_page {
        defmt::info!("Memory: {} pages used", next_free_page);
    } else {
        defmt::warn!("Memory full!");
    }
}
