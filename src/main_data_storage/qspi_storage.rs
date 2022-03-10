pub mod flash_config;
mod identification;
pub mod qspi_driver;

use identification::Identification;

use core::usize;

use alloc::{boxed::Box, vec::Vec};
use stm32l4xx_hal::traits::flash;

use self::qspi_driver::{FlashDriver, QSpiDriver};

use super::PageAccessor;

use qspi_driver::{ClkPin, IO0Pin, IO1Pin, IO2Pin, IO3Pin, NCSPin, QspiError, QUADSPI};

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

pub struct QSPIStorage<CLK, NCS, IO0, IO1, IO2, IO3>
where
    CLK: ClkPin<QUADSPI>,
    NCS: NCSPin<QUADSPI>,
    IO0: IO0Pin<QUADSPI>,
    IO1: IO1Pin<QUADSPI>,
    IO2: IO2Pin<QUADSPI>,
    IO3: IO3Pin<QUADSPI>,
{
    driver: QSpiDriver<CLK, NCS, IO0, IO1, IO2, IO3>,
}

impl<CLK, NCS, IO0, IO1, IO2, IO3> super::storage::Storage
    for QSPIStorage<CLK, NCS, IO0, IO1, IO2, IO3>
where
    CLK: ClkPin<QUADSPI>,
    NCS: NCSPin<QUADSPI>,
    IO0: IO0Pin<QUADSPI>,
    IO1: IO1Pin<QUADSPI>,
    IO2: IO2Pin<QUADSPI>,
    IO3: IO3Pin<QUADSPI>,
{
    fn select_page(&mut self, page: u32) -> Result<Box<dyn PageAccessor>, ()> {
        Err::<Box<dyn PageAccessor>, ()>(())
    }

    fn flash_erease(&mut self) -> Result<(), flash::Error> {
        self.driver.erase().map_err(|_| flash::Error::Failure)
    }

    fn flash_size(&mut self) -> usize {
        self.driver.get_capacity()
    }

    fn flash_size_pages(&mut self) -> u32 {
        self.flash_size() as u32 / self.flash_page_size()
    }

    fn flash_page_size(&mut self) -> u32 {
        // На сколько я понимаю, это минимальный размер который можно стереть за раз
        // можно будет поэкспериментировать с большим размером
        4096
    }
}

impl<CLK, NCS, IO0, IO1, IO2, IO3> QSPIStorage<CLK, NCS, IO0, IO1, IO2, IO3>
where
    CLK: ClkPin<QUADSPI>,
    NCS: NCSPin<QUADSPI>,
    IO0: IO0Pin<QUADSPI>,
    IO1: IO1Pin<QUADSPI>,
    IO2: IO2Pin<QUADSPI>,
    IO3: IO3Pin<QUADSPI>,
{
    pub fn init(
        qspi: qspi_stm32lx3::qspi::Qspi<(CLK, NCS, IO0, IO1, IO2, IO3)>,
        qspi_base_clock_speed: stm32l4xx_hal::time::Hertz,
    ) -> Result<Self, QspiError> {
        let driver = QSpiDriver::init(qspi, qspi_base_clock_speed)?;
        Ok(Self { driver })
    }
}
