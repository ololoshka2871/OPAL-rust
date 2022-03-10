use qspi_stm32lx3::qspi::{QspiError, QspiMode, QspiReadCommand, QspiWriteCommand};

use crate::main_data_storage::qspi_storage::qspi_driver::{FlashDriver, Opcode};

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

    pub struct EnchantedVolatileRegisterCommands: u8 {
        /// Transfer configuration register write
        const WRITE = 0x61;
        /// Transfer configuration register read
        const READ = 0x65;
    }

    pub struct EnchantedVolatileRegisterBits: u8 {
        /// Output driver strength
        const OutputDriver_30ohms = 0b111;
        const OutputDriver_20ohms = 0b101;
        const OutputDriver_45ohms = 0b011;
        const OutputDriver_90ohms = 0b001;

        const reserved = 1 << 3;

        const EnableHoldReset = 1 << 4;
        /// Disable Double data rate
        const DisableDDR = 1 << 5;
        /// Disable Dual IO
        const DisableDIO = 1 << 6;
        /// Disable Quard IO
        const DisableQIO = 1 << 7;
    }
}

impl Default for EnchantedVolatileRegisterBits {
    fn default() -> Self {
        Self {
            bits: (EnchantedVolatileRegisterBits::OutputDriver_30ohms
                | EnchantedVolatileRegisterBits::reserved
                | EnchantedVolatileRegisterBits::EnableHoldReset
                | EnchantedVolatileRegisterBits::DisableDDR
                | EnchantedVolatileRegisterBits::DisableDIO
                | EnchantedVolatileRegisterBits::DisableQIO)
                .bits(),
        }
    }
}

pub fn init(driver: &mut dyn FlashDriver) -> Result<(), QspiError> {
    // status
    let get_status = QspiReadCommand {
        instruction: Some((Opcode::ReadStatus as u8, QspiMode::SingleChannel)),
        address: None,
        alternative_bytes: None,
        dummy_cycles: 0,
        data_mode: QspiMode::SingleChannel,
        receive_length: 3,
        double_data_rate: false,
    };
    let mut status_arr = [0; 1];
    driver.raw_read(get_status, &mut status_arr)?;

    defmt::debug!(
        "Flash status: {}",
        defmt::Debug2Format(&crate::support::hex_slice::HexSlice(status_arr))
    );

    // Flags
    let get_flags = QspiReadCommand {
        instruction: Some((
            EnchantedVolatileRegisterCommands::WRITE.bits(),
            QspiMode::SingleChannel,
        )),
        address: None,
        alternative_bytes: None,
        dummy_cycles: 0,
        data_mode: QspiMode::SingleChannel,
        receive_length: 3,
        double_data_rate: false,
    };
    let mut flags_arr = [0; 1];
    driver.raw_read(get_flags, &mut flags_arr)?;

    defmt::debug!(
        "Flash flags: {}",
        defmt::Debug2Format(&crate::support::hex_slice::HexSlice(flags_arr))
    );

    // 1. set 4 byte addressing
    if unsafe { !Flags::from_bits_unchecked(flags_arr[0]).contains(Flags::ADDRESSING32) } {
        let qspi_state = {
            let mut qspi_state = EnchantedVolatileRegisterBits::default();
            qspi_state.toggle(
                EnchantedVolatileRegisterBits::EnableHoldReset
                    | EnchantedVolatileRegisterBits::DisableQIO,
            );
            [qspi_state.bits(); 1]
        };
        let set_32_bit_adresing = QspiWriteCommand {
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

        driver.raw_write(set_32_bit_adresing)?;

        // verify
        let verify_32_bit_adresing = QspiReadCommand {
            instruction: Some((
                EnchantedVolatileRegisterCommands::WRITE.bits(),
                QspiMode::QuadChannel,
            )),
            address: None,
            alternative_bytes: None,
            dummy_cycles: 0,
            data_mode: QspiMode::QuadChannel,
            receive_length: 3,
            double_data_rate: false,
        };
        let mut result = [0; 1];
        driver.raw_read(verify_32_bit_adresing, &mut result)?;

        if unsafe {
            EnchantedVolatileRegisterBits::from_bits_unchecked(result[0]).contains(
                EnchantedVolatileRegisterBits::EnableHoldReset
                    | EnchantedVolatileRegisterBits::DisableDDR
                    | EnchantedVolatileRegisterBits::DisableQIO,
            )
        } {
            defmt::error!("Failed to configure QSPI mode.");
            return Err(QspiError::Unknown);
        }
    }

    Ok(())
}
