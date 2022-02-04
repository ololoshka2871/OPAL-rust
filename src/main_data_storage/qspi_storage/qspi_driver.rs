#[cfg(any(feature = "stm32l433", feature = "stm32l443"))]
pub use qspi_stm32lx3::{qspi, stm32l4x3::QUADSPI};

#[cfg(not(any(feature = "stm32l433", feature = "stm32l443")))]
use stm32l4xx_hal::{qspi, stm32::QUADSPI};

pub use qspi::{
    ClkPin, IO0Pin, IO1Pin, IO2Pin, IO3Pin, NCSPin, Qspi, QspiConfig, QspiError, QspiMode,
    QspiReadCommand,
};

use super::flash_config::FlashConfig;

// https://github.com/jonas-schievink/spi-memory/blob/master/src/series25.rs

#[allow(unused)]
#[repr(u8)]
enum Opcode {
    /// Read the 8-bit legacy device ID.
    ReadDeviceId = 0xAB,
    /// Read the 8-bit manufacturer and device IDs.
    ReadMfDId = 0x90,
    /// Read 16-bit manufacturer ID and 8-bit device ID.
    ReadJedecId = 0x9F,
    /// Set the write enable latch.
    WriteEnable = 0x06,
    /// Clear the write enable latch.
    WriteDisable = 0x04,
    /// Read the 8-bit status register.
    ReadStatus = 0x05,
    /// Write the 8-bit status register. Not all bits are writeable.
    WriteStatus = 0x01,
    Read = 0x03,
    PageProg = 0x02, // directly writes to EEPROMs too
    SectorErase = 0x20,
    BlockErase = 0xD8,
    ChipErase = 0xC7,
}

pub struct QSpiDriver<CLK, NCS, IO0, IO1, IO2, IO3>
where
    CLK: ClkPin<QUADSPI>,
    NCS: NCSPin<QUADSPI>,
    IO0: IO0Pin<QUADSPI>,
    IO1: IO1Pin<QUADSPI>,
    IO2: IO2Pin<QUADSPI>,
    IO3: IO3Pin<QUADSPI>,
{
    qspi: Qspi<(CLK, NCS, IO0, IO1, IO2, IO3)>,
    config: Option<&'static FlashConfig>,
}

impl<CLK, NCS, IO0, IO1, IO2, IO3> QSpiDriver<CLK, NCS, IO0, IO1, IO2, IO3>
where
    CLK: ClkPin<QUADSPI>,
    NCS: NCSPin<QUADSPI>,
    IO0: IO0Pin<QUADSPI>,
    IO1: IO1Pin<QUADSPI>,
    IO2: IO2Pin<QUADSPI>,
    IO3: IO3Pin<QUADSPI>,
{
    pub fn init(
        mut qspi: Qspi<(CLK, NCS, IO0, IO1, IO2, IO3)>,
        qspi_base_clock_speed: stm32l4xx_hal::time::Hertz,
    ) -> Result<Self, QspiError> {
        qspi.apply_config(QspiConfig::default() /*TODO failsafe config*/);

        let mut res = Self { qspi, config: None };

        let id = res.get_jedec_id()?;

        let config = super::flash_config::FLASH_CONFIGS.iter().find_map(|cfg| {
            if cfg.vendor_id == id.mfr_code() && cfg.capacity_code == id.device_id()[1] {
                Some(cfg)
            } else {
                None
            }
        });

        res.config = config;
        if let Some(config) = config {
            res.qspi.apply_config(config.to_qspi_config(qspi_base_clock_speed));
            defmt::info!("Found QSPI flash: {}", config);
            let nid = res.get_jedec_id()?;
            if nid != id {
                defmt::error!("Failed to apply QSPI config, connection lost");
                Err(QspiError::Unknown)
            } else {
                Ok(res)
            }
        } else {
            defmt::error!("Unknown QSPI flash JDEC ID: {}", defmt::Debug2Format(&id));
            Err(QspiError::Unknown)
        }
    }

    pub fn get_jedec_id(&mut self) -> Result<super::Identification, QspiError> {
        let get_id_command = QspiReadCommand {
            instruction: Some((Opcode::ReadJedecId as u8, QspiMode::SingleChannel)),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 0,
            data_mode: QspiMode::SingleChannel,
            receive_length: 3,
            double_data_rate: false,
        };
        let mut id_arr = [0; 3];

        self.qspi.transfer(get_id_command, &mut id_arr)?;

        Ok(super::Identification::from_jedec_id(&id_arr))
    }

    pub fn get_capacity(&self) -> usize {
        unsafe { self.config.unwrap_unchecked().capacity() }
    }

    pub fn erase(&mut self) -> Result<(), QspiError> {
        Err(QspiError::Unknown)
    }
}
