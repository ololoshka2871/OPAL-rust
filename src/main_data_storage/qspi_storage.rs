pub mod flash_config;
mod identification;
pub mod qspi_driver;

use freertos_rust::{Duration, Mutex};
use identification::Identification;
use usbd_scsi::direct_read::DirectReadHack;

use core::usize;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use stm32l4xx_hal::traits::flash;

use self::qspi_driver::{FlashDriver, QSpiDriver};

use super::PageAccessor;

use qspi_driver::{ClkPin, IO0Pin, IO1Pin, IO2Pin, IO3Pin, NCSPin, QspiError, QUADSPI};

pub struct QSPIFlashPageAccessor<'a> {
    driver: Arc<Mutex<Box<dyn FlashDriver + 'a>>>,
    ptr: *mut u8,
}

impl<'a> PageAccessor for QSPIFlashPageAccessor<'a> {
    fn write(&mut self, data: Vec<u8>) -> Result<(), flash::Error> {
        //self.driver.set_memory_mapping_mode(false).unwrap();
        Err(flash::Error::Failure)
    }

    fn read_to(&self, offset: usize, dest: &mut [u8]) {
        if let Ok(mut guard) = self.driver.lock(Duration::infinite()) {
            guard.set_memory_mapping_mode(true).unwrap();

            unsafe {
                core::ptr::copy_nonoverlapping(self.ptr.add(offset), dest.as_mut_ptr(), dest.len())
            };
        } else {
            unreachable!()
        }
    }

    fn map_to_mem(&self, offset: usize) -> DirectReadHack {
        if let Ok(mut guard) = self.driver.lock(Duration::infinite()) {
            guard.set_memory_mapping_mode(true).unwrap();

            DirectReadHack::new(unsafe { self.ptr.add(offset) })
        } else {
            unreachable!()
        }
    }

    fn erase(&mut self) -> Result<(), flash::Error> {
        //self.driver.set_memory_mapping_mode(false).unwrap();
        Err(flash::Error::Failure)
    }
}

pub struct QSPIStorage<'a> {
    driver: Arc<Mutex<Box<dyn FlashDriver + 'a>>>,
}

impl<'a> super::storage::Storage<'a> for QSPIStorage<'a> {
    fn select_page(&mut self, page: u32) -> Result<Box<dyn PageAccessor + 'a>, flash::Error> {
        const QSPI_MEMORY_MAPPED_REGION: *mut u8 = 0x90000000 as *mut u8;

        let full_adress = (page * self.flash_page_size()) as usize;
        let addr24 = full_adress & 0x00FFFFFF;

        if let Ok(mut guard) = self.driver.lock(Duration::infinite()) {
            if let Err(_) = guard.set_addr_extender((full_adress >> 24) as u8) {
                return Err(flash::Error::Failure);
            }
        }

        let d: Box<dyn PageAccessor + 'a> = Box::new(QSPIFlashPageAccessor {
            driver: self.driver.clone(),
            ptr: unsafe { QSPI_MEMORY_MAPPED_REGION.add(addr24) },
        });
        Ok(d)
    }

    fn flash_erease(&mut self) -> Result<(), flash::Error> {
        if let Ok(mut guard) = self.driver.lock(Duration::zero()) {
            guard.erase().map_err(|_| flash::Error::Failure)
        } else {
            Err(flash::Error::Busy)
        }
    }

    fn flash_size(&mut self) -> usize {
        if let Ok(guard) = self.driver.lock(Duration::zero()) {
            guard.get_capacity()
        } else {
            0
        }
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

impl<'a> QSPIStorage<'a> {
    pub fn init<CLK, NCS, IO0, IO1, IO2, IO3>(
        qspi: qspi_stm32lx3::qspi::Qspi<(CLK, NCS, IO0, IO1, IO2, IO3)>,
        qspi_base_clock_speed: stm32l4xx_hal::time::Hertz,
    ) -> Result<Self, QspiError>
    where
        CLK: ClkPin<QUADSPI> + 'a,
        NCS: NCSPin<QUADSPI> + 'a,
        IO0: IO0Pin<QUADSPI> + 'a,
        IO1: IO1Pin<QUADSPI> + 'a,
        IO2: IO2Pin<QUADSPI> + 'a,
        IO3: IO3Pin<QUADSPI> + 'a,
    {
        if let Ok(driver) = QSpiDriver::init(qspi, qspi_base_clock_speed) {
            Ok(Self {
                driver: Arc::new(
                    Mutex::new(
                        // этот каст нужен, иначе тип мьютекса неправильный
                        Box::new(driver) as Box<dyn FlashDriver + 'a>,
                    )
                    .unwrap(),
                ),
            })
        } else {
            Err(QspiError::Unknown)
        }
    }
}
