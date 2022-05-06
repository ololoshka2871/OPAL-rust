use qspi_stm32lx3::qspi::{AddressSize, QspiError};
use stm32l4xx_hal::time::Hertz;

use super::qspi_driver::{FlashDriver, QspiConfig};

#[allow(non_snake_case)]
mod MT25QU01GBBB8E12;

pub struct FlashConfig {
    pub vendor_id: u8,
    pub capacity_code: u8,
    pub vendor_name: &'static str,
    pub read_dumy_cycles: u8,
    pub write_dumy_cycles: u8,
    pub write_max_bytes: usize,

    pub wake_up_command_code: u8,
    pub enter_deep_sleep_command_code: u8,

    pub is_busy: fn(driver: &mut dyn FlashDriver, qspi_mode: bool) -> Result<bool, QspiError>,
    pub check_write_ok: fn(driver: &mut dyn FlashDriver, qspi_mode: bool) -> Result<(), QspiError>,

    pub chip_erase: fn(driver: &mut dyn FlashDriver, qspi_mode: bool) -> Result<(), QspiError>,

    address_size: AddressSize,
    qspi_flash_size_code: u8, // using 24bit addressing, 16 MB max per page
    qspi_max_freq: Hertz,

    flash_prepare_qspi: Option<fn(driver: &mut dyn FlashDriver) -> Result<(), QspiError>>,
    special_qspi_config: Option<fn(cfg: &mut QspiConfig) -> Result<(), QspiError>>,
    flash_finalise_config: Option<fn(driver: &mut dyn FlashDriver) -> Result<(), QspiError>>,
}

impl FlashConfig {
    pub fn capacity(&self) -> usize {
        2usize.pow(self.qspi_flash_size_code as u32 + 1)
    }

    pub fn configure(
        &self,
        driver: &mut dyn FlashDriver,
        qspi_base_clock_speed: Hertz,
    ) -> Result<(), QspiError> {
        fn call_if_not_none(
            driver: &mut dyn FlashDriver,
            f: Option<fn(driver: &mut dyn FlashDriver) -> Result<(), QspiError>>,
        ) -> Result<(), QspiError> {
            if let Some(f) = f {
                f(driver)
            } else {
                Ok(())
            }
        }

        call_if_not_none(driver, self.flash_prepare_qspi)?;

        let mut cfg = QspiConfig::default()
            .clock_prescaler(core::cmp::max(
                1,
                (qspi_base_clock_speed.0 / self.qspi_max_freq.0) as u8,
            ))
            .clock_mode(qspi_stm32lx3::qspi::ClockMode::Mode3)
            .flash_size(core::cmp::min(self.qspi_flash_size_code, 23))
            .address_size(self.address_size)
            .chip_select_high_time(
                core::cmp::min((qspi_base_clock_speed.0 / 10_000_000) as u8, 8), // max 8
            )
            .qpi_mode(true);

        if let Some(f) = self.special_qspi_config {
            f(&mut cfg)?;
        }
        driver.apply_qspi_config(cfg);

        call_if_not_none(driver, self.flash_finalise_config)
    }
}

impl defmt::Format for FlashConfig {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "Vendor: {}, capacity {} bytes",
            self.vendor_name,
            self.capacity()
        )
    }
}

static VENDORS: [&str; 1] = ["Micron"];

pub static FLASH_CONFIGS: [FlashConfig; 1] = [
    // MT25QU01GBBB8E12
    FlashConfig {
        vendor_id: 0x20,
        capacity_code: 0x21, // 1Gb
        vendor_name: VENDORS[0],

        read_dumy_cycles: 10,
        write_dumy_cycles: 0,

        write_max_bytes: 256, // QUAD INPUT FAST PROGRAM command
        wake_up_command_code: MT25QU01GBBB8E12::DeepSleepCmd::WAKE_UP_COMMAND_CODE.bits(),
        enter_deep_sleep_command_code:
            MT25QU01GBBB8E12::DeepSleepCmd::ENTER_DEEP_SLEEP_COMMAND_CODE.bits(),
        is_busy: MT25QU01GBBB8E12::is_busy, // QUAD INPUT/OUTPUT FAST READ command (factory-default)
        check_write_ok: MT25QU01GBBB8E12::check_write_ok,
        address_size: AddressSize::Addr24Bit,

        qspi_flash_size_code: 26,
        qspi_max_freq: Hertz(20_000_000),
        flash_prepare_qspi: Some(MT25QU01GBBB8E12::flash_prepare_qspi),
        special_qspi_config: None,
        flash_finalise_config: Some(MT25QU01GBBB8E12::flash_finalise_config),
        chip_erase: MT25QU01GBBB8E12::chip_erase,
    },
];
