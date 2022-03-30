pub mod data_page;
pub mod diff_writer;
pub mod write_controller;

pub mod storage;

pub(crate) mod header_printer;
mod qspi_storage;

use core::sync::atomic::AtomicBool;
use lazy_static::lazy_static;

use alloc::boxed::Box;
use freertos_rust::{Duration, FreeRtosError, Mutex, Task, TaskPriority};

use stm32l4xx_hal::traits::flash;

use qspi_storage::qspi_driver::{ClkPin, IO0Pin, IO1Pin, IO2Pin, IO3Pin, NCSPin, QUADSPI};

#[derive(Copy, Clone)]
pub enum MemoryState {
    Undefined,
    PartialUsed(u32),
    FullUsed,
}

static mut STORAGE_IMPL: Option<Box<dyn storage::Storage + 'static>> = None;
static mut ERASE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref NEXT_EMPTY_PAGE: Mutex<MemoryState> = Mutex::new(MemoryState::Undefined).unwrap();
}

pub trait PageAccessor {
    fn write(&mut self, data: &[u8]) -> Result<(), flash::Error>;
    fn read_to(&self, offset: usize, dest: &mut [u8]);
    fn map_to_mem(&self, offset: usize) -> usbd_scsi::direct_read::DirectReadHack;
    fn erase(&mut self) -> Result<(), flash::Error>;
}

pub fn memory_state() -> MemoryState {
    if let Ok(guard) = NEXT_EMPTY_PAGE.lock(Duration::zero()) {
        *guard
    } else {
        MemoryState::Undefined
    }
}

pub fn is_erase_in_progress() -> bool {
    unsafe { ERASE_IN_PROGRESS.load(core::sync::atomic::Ordering::Relaxed) }
}

pub fn flash_erease() -> Result<(), FreeRtosError> {
    if !is_erase_in_progress() {
        unsafe { ERASE_IN_PROGRESS.store(true, core::sync::atomic::Ordering::Relaxed) };
        let _ = Task::new()
            .name("FlashClr")
            .priority(TaskPriority(crate::config::FLASH_CLEANER_PRIO))
            .start(move |_| {
                let _ = lock_storage(|s| {
                    if let Err(e) = s.flash_erease() {
                        defmt::error!("Flash erase error: {}", defmt::Debug2Format(&e));
                    } else {
                        defmt::info!("Flash erased succesfilly");
                    }
                });

                unsafe { ERASE_IN_PROGRESS.store(false, core::sync::atomic::Ordering::Relaxed) };
            })?;

        Ok(())
    } else {
        Err(FreeRtosError::QueueFull)
    }
}

pub fn find_next_empty_page(start: u32) -> Option<u32> {
    if let Ok(guard) = NEXT_EMPTY_PAGE.lock(Duration::ms(1)) {
        match *guard {
            MemoryState::PartialUsed(next_free_page) => {
                if start <= next_free_page {
                    return Some(next_free_page);
                }
            }
            MemoryState::FullUsed => return None,
            _ => {}
        }
    }

    if start < flash_size_pages() {
        for p in start..flash_size_pages() {
            let accessor = unsafe { select_page(p).unwrap_unchecked() };
            let header_blockchain = unsafe {
                core::ptr::read_volatile(
                    accessor
                        .map_to_mem(0)
                        .pointer::<self_recorder_packet::DataPacketHeader>(),
                )
            };

            // признаком того, что флешка стерта является то, что там везде FF
            if core::cmp::min(
                header_blockchain.this_block_id,
                header_blockchain.prev_block_id,
            ) == u32::MAX
            {
                if let Ok(mut guard) = NEXT_EMPTY_PAGE.lock(Duration::zero()) {
                    *guard = MemoryState::PartialUsed(p);
                }
                return Some(p);
            }
        }
    }
    if let Ok(mut guard) = NEXT_EMPTY_PAGE.lock(Duration::zero()) {
        *guard = MemoryState::FullUsed;
    }
    None
}

pub fn select_page<'a>(page: u32) -> Result<Box<dyn PageAccessor + 'a>, flash::Error> {
    lock_storage(|s| s.select_page(page)).unwrap_or_else(|e| Err(e))
}

pub fn flash_page_size() -> u32 {
    lock_storage(|s| s.flash_page_size()).unwrap_or(0)
}

pub fn flash_size() -> usize {
    lock_storage(|s| s.flash_size()).unwrap_or(0)
}

pub fn flash_size_pages() -> u32 {
    lock_storage(|s| s.flash_size_pages()).unwrap_or(0)
}

pub(crate) fn init<CLK, NCS, IO0, IO1, IO2, IO3>(
    qspi: qspi_stm32lx3::qspi::Qspi<(CLK, NCS, IO0, IO1, IO2, IO3)>,
    sys_clk: stm32l4xx_hal::time::Hertz,
) where
    CLK: ClkPin<QUADSPI> + 'static,
    NCS: NCSPin<QUADSPI> + 'static,
    IO0: IO0Pin<QUADSPI> + 'static,
    IO1: IO1Pin<QUADSPI> + 'static,
    IO2: IO2Pin<QUADSPI> + 'static,
    IO3: IO3Pin<QUADSPI> + 'static,
{
    if let Ok(s) = qspi_storage::QSPIStorage::init(qspi, sys_clk) {
        unsafe { STORAGE_IMPL.replace(Box::new(s)) };

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

fn lock_storage<T, F>(f: F) -> Result<T, flash::Error>
where
    F: FnOnce(&mut dyn storage::Storage<'static>) -> T,
{
    if let Some(s) = unsafe { &mut STORAGE_IMPL } {
        Ok(f(s.as_mut()))
    } else {
        Err(flash::Error::Illegal)
    }
}
