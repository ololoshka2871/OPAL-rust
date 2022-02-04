use core::sync::atomic::AtomicBool;

use alloc::{boxed::Box, vec::Vec};
use freertos_rust::{FreeRtosError, Task, TaskPriority};

use stm32l4xx_hal::traits::flash;

pub mod data_page;
pub mod diff_writer;
pub mod write_controller;

pub mod storage;

pub(crate) mod header_printer;
//mod internal_storage;
mod qspi_storage;

use qspi_storage::qspi_driver::{ClkPin, IO0Pin, IO1Pin, IO2Pin, IO3Pin, NCSPin, QUADSPI};

enum MemoryState {
    Undefined,
    PartialUsed(u32),
    FullUsed,
}

static mut NEXT_EMPTY_PAGE: MemoryState = MemoryState::Undefined;
static mut ERASER_THREAD: (Option<Task>, AtomicBool) = (None, AtomicBool::new(false));
static mut STORAGE_IMPL: Option<Box<dyn storage::Storage>> = None;

pub trait PageAccessor {
    fn write(&mut self, data: Vec<u8>) -> Result<(), flash::Error>;
    fn read_to(&self, offset: usize, dest: &mut [u8]);
    fn erase(&mut self) -> Result<(), flash::Error>;
}

pub fn is_erase_in_progress() -> bool {
    if unsafe { ERASER_THREAD.1.load(core::sync::atomic::Ordering::Relaxed) } {
        true
    } else {
        if unsafe { ERASER_THREAD.0.is_some() } {
            unsafe { ERASER_THREAD.0 = None };
        }
        false
    }
}

pub fn flash_erease() -> Result<(), FreeRtosError> {
    if !is_erase_in_progress() {
        let task = Task::new()
            .name("FlashClr")
            .priority(TaskPriority(crate::config::FLASH_CLEANER_PRIO))
            .start(move |_| {
                if let Some(s) = unsafe { &mut STORAGE_IMPL } {
                    if let Err(e) = s.flash_erease() {
                        defmt::error!("Flash erase error: {}", defmt::Debug2Format(&e));
                    } else {
                        defmt::info!("Flash erased succesfilly");
                    }
                }

                unsafe {
                    NEXT_EMPTY_PAGE = MemoryState::Undefined;
                    ERASER_THREAD
                        .1
                        .store(false, core::sync::atomic::Ordering::Relaxed);
                };
            })?;

        unsafe {
            ERASER_THREAD.0 = Some(task);
            ERASER_THREAD
                .1
                .store(true, core::sync::atomic::Ordering::Relaxed)
        };

        Ok(())
    } else {
        Err(FreeRtosError::OutOfMemory)
    }
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

pub fn select_page(page: u32) -> Result<Box<dyn PageAccessor>, ()> {
    unsafe {
        STORAGE_IMPL
            .as_deref_mut()
            .map_or(Err(()), |s| s.select_page(page))
    }
}

pub fn flash_page_size() -> u32 {
    unsafe {
        STORAGE_IMPL
            .as_deref_mut()
            .map_or(0, |s| s.flash_page_size())
    }
}

pub fn flash_size() -> usize {
    unsafe { STORAGE_IMPL.as_deref_mut().map_or(0, |s| s.flash_size()) }
}

pub fn flash_size_pages() -> u32 {
    unsafe {
        STORAGE_IMPL
            .as_deref_mut()
            .map_or(0, |s| s.flash_size_pages())
    }
}

pub(crate) fn init<CLK, NCS, IO0, IO1, IO2, IO3>(
    qspi: qspi_stm32lx3::qspi::Qspi<(CLK, NCS, IO0, IO1, IO2, IO3)>,
    qspi_base_clock_speed: stm32l4xx_hal::time::Hertz,
) where
    CLK: ClkPin<QUADSPI> + 'static,
    NCS: NCSPin<QUADSPI> + 'static,
    IO0: IO0Pin<QUADSPI> + 'static,
    IO1: IO1Pin<QUADSPI> + 'static,
    IO2: IO2Pin<QUADSPI> + 'static,
    IO3: IO3Pin<QUADSPI> + 'static,
{
    if let Ok(s) = qspi_storage::QSPIStorage::init(qspi, qspi_base_clock_speed) {
        unsafe { STORAGE_IMPL = Some(Box::new(s)) };

        let next_free_page = find_next_empty_page(0);
        if let Some(next_free_page) = next_free_page {
            defmt::info!("Memory: {} pages used", next_free_page);
        } else {
            defmt::warn!("Memory full!");
        }
    } else {
        defmt::error!("Failed to initialise QSPI flash! storage blocked!");
    }
}
