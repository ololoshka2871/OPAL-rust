use freertos_rust::Duration;
use qspi_stm32lx3::qspi::{QspiError, QspiMode, QspiReadCommand, QspiWriteCommand};

use crate::main_data_storage::qspi_storage::qspi_driver::FlashDriver;

bitflags::bitflags! {
    /// Status register bits.
    pub struct Flags: u8 {
        /// use 4 Byte absolute addressing
        const ADDRESSING32 = 1 << 0;
        /// Protection error while perform last operation
        const PROTECTION_ERROR = 1 << 1;
        /// Indicates whether a PROGRAM operation has been or is going to be suspended.
        const PROG_SUSPEND = 1 << 2;
        /// Indicates whether a PROGRAM operation has succeeded or failed. It indicates, also, whether a CRC check has succeeded or failed.
        const PROGRAM_ERR = 1<<4;
        /// Indicates whether an ERASE operation has succeeded or failed.
        const ERASE_ERR = 1<<5;
        ///  Indicates whether an ERASE operation has been or is going to be suspended.
        const ERASE_SUSPEND = 1 << 6;
        /// Indicates whether one of the following command cycles is in progress: WRITE STATUS REGISTER, WRITE NONVOLATILE CONFIGURATION REGISTER, PROGRAM, or ERASE.
        const PROG_OR_ERASE = 1 << 7;
    }

    pub struct VolatileRegisterCommands: u8 {
        /// Transfer configuration register write
        const WRITE = 0x81;
        /// Transfer configuration register read
        const READ = 0x85;
    }

    pub struct EnchantedVolatileRegisterCommands: u8 {
        /// Transfer enchanted configuration register write
        const WRITE = 0x61;
        /// Transfer enchanted configuration register read
        const READ = 0x65;
    }

    pub struct FlagStatusRegisterCommands: u8 {
        /// Read status regiaster
        const READ = 0x70;
    }

    pub struct StatusFlagsRegisterBits: u8 {
        const PROG_OR_ERASE_CTRL = 1 << 7; // 0 - busy
        const ERASE_SUSPEND = 1 << 6;
        const ERASE_FAILURE = 1 << 5;
        const PROGRAM_FAILURE = 1 << 4;
        const RESERVED = 1 << 3;
        const PROGRAMM_SUSPEND = 1 << 2;
        const PROTECTION_FAILURE = 1 << 1;
        const FOR_BYTE_ADDRESSING = 1 << 0;
    }

    pub struct EnchantedVolatileRegisterBits: u8 {
        /// Output driver strength
        const OUTPUT_DRIVER_30_OHMS = 0b111;
        const OUTPUT_DRIVER_20_OHMS = 0b101;
        const OUTPUT_DRIVER_45_OHMS = 0b011;
        const OUTPUT_DRIVER_90_OHMS = 0b001;

        const RESERVED = 1 << 3;

        const ENABLE_HOLD_RESET = 1 << 4;
        /// Disable Double data rate
        const DISABLE_DDR = 1 << 5;
        /// Disable Dual IO
        const DISABLE_DIO = 1 << 6;
        /// Disable Quard IO
        const DISABLE_QIO = 1 << 7;
    }

    pub struct DeepSleepCmd: u8 {
        const ENTER_DEEP_SLEEP_COMMAND_CODE = 0xB9;
        const WAKE_UP_COMMAND_CODE = 0xab;
    }

    pub struct EraseCommnds: u8 {
        // effects on 128M
        const DIE_ERASE = 0xC4;
    }

    pub struct DieConfig : usize {
        const DIE_COUNT = 2;
        const DIE_SIZE = 128 * 1024 * 1024;
    }
}

impl Default for EnchantedVolatileRegisterBits {
    fn default() -> Self {
        // factory defaults
        Self {
            bits: (EnchantedVolatileRegisterBits::OUTPUT_DRIVER_30_OHMS
                | EnchantedVolatileRegisterBits::RESERVED
                | EnchantedVolatileRegisterBits::ENABLE_HOLD_RESET
                | EnchantedVolatileRegisterBits::DISABLE_DDR
                | EnchantedVolatileRegisterBits::DISABLE_DIO
                | EnchantedVolatileRegisterBits::DISABLE_QIO)
                .bits(),
        }
    }
}

fn get_non_volatile_cfg_reg(
    driver: &mut dyn FlashDriver,
    qspi_mode: bool,
) -> Result<EnchantedVolatileRegisterBits, QspiError> {
    let mode = if qspi_mode {
        QspiMode::QuadChannel
    } else {
        QspiMode::SingleChannel
    };
    let get_non_volatile_cfg_reg_cmd = QspiReadCommand {
        instruction: Some((
            EnchantedVolatileRegisterCommands::READ.bits(),
            //VolatileRegisterCommands::READ.bits(),
            mode,
        )),
        address: None,
        alternative_bytes: None,
        dummy_cycles: 0, // Comand set table - 0 in any mode
        data_mode: mode,
        receive_length: 1,
        double_data_rate: false,
    };

    let mut result = [0; 1];
    driver.raw_read(get_non_volatile_cfg_reg_cmd, &mut result)?;
    Ok(unsafe { EnchantedVolatileRegisterBits::from_bits_unchecked(result[0]) })
}

pub fn flash_prepare_qspi(driver: &mut dyn FlashDriver) -> Result<(), QspiError> {
    let cfg = get_non_volatile_cfg_reg(driver, false)?;
    defmt::debug!("Current volatile enc. cfg: 0x{:X}", cfg.bits());

    // 1. Write enable
    driver.write_enable()?;

    // 2. enable QSPI
    let qspi_state = {
        let qspi_state = EnchantedVolatileRegisterBits::default()
            ^ (EnchantedVolatileRegisterBits::ENABLE_HOLD_RESET
                | EnchantedVolatileRegisterBits::DISABLE_QIO);
        [qspi_state.bits(); 1]
    };
    let set_qspi_cmd = QspiWriteCommand {
        instruction: Some((
            EnchantedVolatileRegisterCommands::WRITE.bits(),
            QspiMode::SingleChannel,
        )),
        address: None,
        alternative_bytes: None,
        dummy_cycles: 0,
        data: Some((&qspi_state, QspiMode::SingleChannel)),
        double_data_rate: false,
    };

    driver.raw_write(set_qspi_cmd)
}

pub fn flash_finalise_config(driver: &mut dyn FlashDriver) -> Result<(), QspiError> {
    // 3. verify QSPI mode works
    let result = get_non_volatile_cfg_reg(driver, true)?;

    if result
        != EnchantedVolatileRegisterBits::default()
            ^ (EnchantedVolatileRegisterBits::ENABLE_HOLD_RESET
                | EnchantedVolatileRegisterBits::DISABLE_QIO)
    {
        defmt::error!(
            "Failed to configure QSPI mode. (cfg: 0x{:X})",
            result.bits()
        );
        return Err(QspiError::Unknown);
    }

    // 4. Factory defaults
    // Adress size - 3 bytes
    // DDR - disabled
    // dummy cycles - default

    Ok(())
}

fn read_status(
    driver: &mut dyn FlashDriver,
    qspi_mode: bool,
) -> Result<StatusFlagsRegisterBits, QspiError> {
    let mode = if qspi_mode {
        QspiMode::QuadChannel
    } else {
        QspiMode::SingleChannel
    };
    let get_flag_status_reg_cmd = QspiReadCommand {
        instruction: Some((FlagStatusRegisterCommands::READ.bits(), mode)),
        address: None,
        alternative_bytes: None,
        dummy_cycles: 0, // internal command - no dummy
        data_mode: mode,
        receive_length: 1,
        double_data_rate: false,
    };

    let mut result = [0; 1];
    driver.raw_read(get_flag_status_reg_cmd, &mut result)?;
    Ok(unsafe { StatusFlagsRegisterBits::from_bits_unchecked(result[0]) })
}

pub fn is_busy(driver: &mut dyn FlashDriver, qspi_mode: bool) -> Result<bool, QspiError> {
    let status = read_status(driver, qspi_mode)?;
    Ok(!(status.contains(StatusFlagsRegisterBits::PROG_OR_ERASE_CTRL)))
}

pub fn check_write_ok(driver: &mut dyn FlashDriver, qspi_mode: bool) -> Result<(), QspiError> {
    let status = read_status(driver, qspi_mode)?;
    if status.contains(StatusFlagsRegisterBits::PROGRAM_FAILURE)
        | status.contains(StatusFlagsRegisterBits::PROTECTION_FAILURE)
    {
        Err(QspiError::Unknown)
    } else {
        Ok(())
    }
}

pub fn chip_erase(driver: &mut dyn FlashDriver, qspi_mode: bool) -> Result<(), QspiError> {
    for die in 0..DieConfig::DIE_COUNT.bits() {
        let full_adress = (die * DieConfig::DIE_SIZE.bits()) as u32;

        driver.write_enable()?;
        driver.set_addr_extender((full_adress >> 24) as u8)?;

        driver.write_enable()?;
        let erase_cmd = QspiWriteCommand {
            instruction: Some((EraseCommnds::DIE_ERASE.bits(), QspiMode::QuadChannel)),
            address: Some((full_adress & 0x00FFFFFF, QspiMode::QuadChannel)),
            alternative_bytes: None,
            dummy_cycles: 0,
            data: None,
            double_data_rate: false,
        };
        driver.raw_write(erase_cmd)?;

        while is_busy(driver, qspi_mode)? {
            /* wait write complead */
            freertos_rust::CurrentTask::delay(Duration::ticks(10));
        }
    }
    Ok(())
}
