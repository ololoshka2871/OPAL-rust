use core::usize;

use alloc::{sync::Arc, vec::Vec};
use stm32l4xx_hal::traits::flash;

use qspi_stm32lx3::{
    qspi::{ClkPin, IO0Pin, IO1Pin, IO2Pin, IO3Pin, NCSPin},
    stm32l4x3::QUADSPI,
};

use super::PageAccessor;

pub struct QSPIFlashPageAccessor {
    ptr: *mut u8,
}

impl PageAccessor for QSPIFlashPageAccessor {
    fn write(&mut self, data: Vec<u8>) -> Result<(), flash::Error> {
        Err(flash::Error::Failure)
    }

    fn read_to(&self, offset: usize, dest: &mut [u8]) {}

    fn erase(&mut self) -> Result<(), flash::Error> {
        Err(flash::Error::Failure)
    }
}

pub fn select_page(page: u32) -> Result<impl PageAccessor, ()> {
    Err::<QSPIFlashPageAccessor, ()>(())
}

pub fn flash_erease() -> Result<(), flash::Error> {
    Err(flash::Error::Failure)
}

pub fn flash_size() -> usize {
    0
}

pub fn flash_size_pages() -> u32 {
    0
}

pub fn flash_page_size() -> u32 {
    0
}

pub(crate) fn init<CLK, NCS, IO0, IO1, IO2, IO3>(
    qspi: qspi_stm32lx3::qspi::Qspi<(CLK, NCS, IO0, IO1, IO2, IO3)>,
    qspi_base_clock_speed: stm32l4xx_hal::time::Hertz,
) where
    CLK: ClkPin<QUADSPI>,
    NCS: NCSPin<QUADSPI>,
    IO0: IO0Pin<QUADSPI>,
    IO1: IO1Pin<QUADSPI>,
    IO2: IO2Pin<QUADSPI>,
    IO3: IO3Pin<QUADSPI>,
{
}
