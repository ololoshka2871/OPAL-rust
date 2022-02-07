use super::qspi_driver::QspiConfig;

pub struct FlashConfig {
    pub vendor_id: u8,
    pub capacity_code: u8,
    pub vendor_name: &'static str,
}

impl FlashConfig {
    pub fn capacity(&self) -> usize {
        let pow = self.capacity_code - 1;
        let bits = (1usize << pow) * 1024;
        bits / 8
    }

    pub fn to_qspi_config(&self, qspi_base_clock_speed: stm32l4xx_hal::time::Hertz) -> QspiConfig {
        QspiConfig::default().clock_prescaler(1)
    }
}

impl defmt::Format for FlashConfig {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            r#"Vendor: {}, capacity {} bytes"#,
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
        capacity_code: 21,
        vendor_name: VENDORS[0],
    },
];
