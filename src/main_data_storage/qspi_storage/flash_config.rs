use qspi_stm32lx3::qspi::{AddressSize, QspiError};

use super::qspi_driver::QspiConfig;

#[allow(non_snake_case)]
mod MT25QU01GBBB8E12;

pub struct FlashConfig {
    pub vendor_id: u8,
    pub capacity_code: u8,
    pub vendor_name: &'static str,
    pub flash_init: fn(driver: &mut dyn super::qspi_driver::FlashDriver) -> Result<(), QspiError>,
    address_size: AddressSize,
    qspi_flash_size_code: u8,
}

impl FlashConfig {
    pub fn capacity(&self) -> usize {
        2usize.pow(self.qspi_flash_size_code as u32 + 1)
    }

    pub fn to_qspi_config(&self, qspi_base_clock_speed: stm32l4xx_hal::time::Hertz) -> QspiConfig {
        QspiConfig::default()
            .clock_prescaler(1)
            .clock_mode(qspi_stm32lx3::qspi::ClockMode::Mode3)
            .flash_size(self.qspi_flash_size_code)
            .address_size(self.address_size)
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

        flash_init: MT25QU01GBBB8E12::init,

        // по дефолту включена 3 байтовая адресация, нужно переключение
        address_size: AddressSize::Addr32Bit,
        qspi_flash_size_code: 26,
    },
];
