use qspi_stm32lx3::qspi::QspiWriteCommand;
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
pub enum Opcode {
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
    /// Read 16-bit manufacturer ID and 8-bit device ID. MultiIO
    ReadJedecIdMIO = 0xAF,
    /// Read flags register
    ReadFlagStatus = 0x70,
    /// QIO fast read 3 byte address
    QIOFastRead = 0xEB,
    /// Write address extander register
    WriteAddrExtanderReg = 0xC5,
}

pub trait FlashDriver {
    fn get_jedec_id(&mut self) -> Result<super::Identification, QspiError>;
    fn get_jedec_id_qio(&mut self) -> Result<super::Identification, QspiError>;
    fn get_capacity(&self) -> usize;
    fn erase(&mut self) -> Result<(), QspiError>;
    fn raw_read(&mut self, command: QspiReadCommand, buffer: &mut [u8]) -> Result<(), QspiError>;
    fn raw_write(&mut self, command: QspiWriteCommand) -> Result<(), QspiError>;
    fn config(&self) -> &FlashConfig;
    fn apply_qspi_config(&mut self, cfg: QspiConfig);
    fn set_memory_mapping_mode(&mut self, enable: bool) -> Result<(), QspiError>;
    fn set_addr_extender(&mut self, extender_value: u8) -> Result<(), QspiError>;
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
    mamory_mapped: bool,
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
        qspi.apply_config(
            QspiConfig::default()
                /* failsafe config */
                .clock_prescaler((qspi_base_clock_speed.0 / 1_000_000) as u8)
                .clock_mode(qspi::ClockMode::Mode3),
        );

        let mut res = Self {
            qspi,
            config: None,
            mamory_mapped: false,
        };

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
            defmt::info!("Found flash: {}", config);

            if let Err(e) = config.configure(&mut res, qspi_base_clock_speed) {
                defmt::error!("Failed to init flash!");
                Err(e)
            } else {
                let newid = res.get_jedec_id_qio()?;
                if newid == id {
                    defmt::info!("Initialised QSPI flash: {}", config);
                    Ok(res)
                } else {
                    defmt::error!("Failed to verify id in QSPI mode");
                    Err(QspiError::Unknown)
                }
            }
        } else {
            defmt::error!("Unknown QSPI flash JDEC ID: {}", defmt::Debug2Format(&id));
            Err(QspiError::Unknown)
        }
    }

    pub fn get_jedec_id_cfg(&mut self, use_qspi: bool) -> Result<super::Identification, QspiError> {
        let get_id_command = QspiReadCommand {
            instruction: if use_qspi {
                Some((Opcode::ReadJedecIdMIO as u8, QspiMode::QuadChannel))
            } else {
                Some((Opcode::ReadJedecId as u8, QspiMode::SingleChannel))
            },
            address: None,
            alternative_bytes: None,
            dummy_cycles: 0,
            data_mode: if use_qspi {
                QspiMode::QuadChannel
            } else {
                QspiMode::SingleChannel
            },
            receive_length: 3,
            double_data_rate: false,
        };
        let mut id_arr = [0; 3];

        self.qspi.transfer(get_id_command, &mut id_arr)?;

        Ok(super::Identification::from_jedec_id(&id_arr))
    }
}

impl<CLK, NCS, IO0, IO1, IO2, IO3> FlashDriver for QSpiDriver<CLK, NCS, IO0, IO1, IO2, IO3>
where
    CLK: ClkPin<QUADSPI>,
    NCS: NCSPin<QUADSPI>,
    IO0: IO0Pin<QUADSPI>,
    IO1: IO1Pin<QUADSPI>,
    IO2: IO2Pin<QUADSPI>,
    IO3: IO3Pin<QUADSPI>,
{
    fn get_jedec_id(&mut self) -> Result<super::Identification, QspiError> {
        self.get_jedec_id_cfg(false)
    }

    fn get_jedec_id_qio(&mut self) -> Result<super::Identification, QspiError> {
        self.get_jedec_id_cfg(true)
    }

    fn get_capacity(&self) -> usize {
        unsafe { self.config.unwrap_unchecked().capacity() }
    }

    fn erase(&mut self) -> Result<(), QspiError> {
        Err(QspiError::Unknown)
    }

    fn raw_read(&mut self, command: QspiReadCommand, buffer: &mut [u8]) -> Result<(), QspiError> {
        self.qspi.transfer(command, buffer)
    }

    fn raw_write(&mut self, command: QspiWriteCommand) -> Result<(), QspiError> {
        self.qspi.write(command)
    }

    fn config(&self) -> &FlashConfig {
        self.config.unwrap()
    }

    fn apply_qspi_config(&mut self, cfg: QspiConfig) {
        self.qspi.apply_config(cfg)
    }

    fn set_memory_mapping_mode(&mut self, enable: bool) -> Result<(), QspiError> {
        if enable == self.mamory_mapped {
            return Ok(());
        }

        if let Some(cfg) = self.config {
            if enable {
                const DUMMY: [u8; 1] = [0];
                let enable_mapping_cmd = QspiWriteCommand {
                    instruction: Some((Opcode::QIOFastRead as u8, QspiMode::QuadChannel)),
                    address: Some((0, QspiMode::QuadChannel)),
                    alternative_bytes: None,
                    dummy_cycles: cfg.qspi_dumy_cycles,
                    data: Some((&DUMMY, QspiMode::QuadChannel)),
                    double_data_rate: false,
                };

                self.qspi.start_memory_mapping(enable_mapping_cmd)?;
            } else {
                self.qspi.abort_transmission();
            }
            self.mamory_mapped = enable;

            Ok(())
        } else {
            Err(QspiError::Unknown)
        }
    }

    fn set_addr_extender(&mut self, extender_value: u8) -> Result<(), QspiError> {
        if self.mamory_mapped {
            self.set_memory_mapping_mode(false)?;
        }

        let data = [extender_value];
        let set_extender_cmd = QspiWriteCommand {
            instruction: Some((Opcode::WriteAddrExtanderReg as u8, QspiMode::QuadChannel)),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 0, // internal register, no wait
            data: Some((&data, QspiMode::QuadChannel)),
            double_data_rate: false,
        };

        self.qspi.write(set_extender_cmd)
    }
}
