use core::convert::TryInto;

use usbd_scsi::{BlockDevice, BlockDeviceError};

pub struct Storage {}

static mut DATA: [u8; 512] = [0_u8; 512];

impl BlockDevice for Storage {
    const BLOCK_BYTES: usize = 512;

    fn read_block(&self, _lba: u32, block: &mut [u8]) -> Result<(), BlockDeviceError> {
        let block: &mut [u8; 512] = block
            .try_into()
            .map_err(|_e| BlockDeviceError::InvalidAddress)?;

        unsafe {
            core::ptr::copy_nonoverlapping(DATA.as_ptr(), block.as_mut_ptr(), 512);
        }
        Ok(())
    }

    fn write_block(&mut self, _lba: u32, block: &[u8]) -> Result<(), BlockDeviceError> {
        let block: &[u8; 512] = block
            .try_into()
            .map_err(|_e| BlockDeviceError::InvalidAddress)?;

        unsafe {
            core::ptr::copy_nonoverlapping(block.as_ptr(), DATA.as_mut_ptr(), 512);
        }
        Ok(())
    }

    fn max_lba(&self) -> u32 {
        0 // Это не размер а максимальный номер блока по 512 байт
    }
}
