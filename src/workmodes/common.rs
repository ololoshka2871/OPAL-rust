
pub fn print_clock_config(clocks: &Option<stm32l4xx_hal::rcc::Clocks>, usb_state: &str) {
    if let Some(clocks) = clocks {
        defmt::info!(
            "Clock config: CPU={}, pclk1={}, pclk2={}, USB: {}",
            clocks.sysclk().0,
            clocks.pclk1().0,
            clocks.pclk2().0,
            usb_state
        );
    } else {
        defmt::error!("System clock not configures yet");
    }
}
