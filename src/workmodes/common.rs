use freertos_rust::{Duration, DurationTicks};
use stm32l4xx_hal::time::{Hertz, KiloHertz, MegaHertz};

pub static HSE_FREQ: MegaHertz = MegaHertz(12);

pub fn calc_monitoring_period<D: DurationTicks, F: Into<Hertz>>(period: D, sysclk: F) -> Duration {
    let in_freq_khz: KiloHertz = crate::workmodes::common::HSE_FREQ.into();
    let fcpu_khz  = KiloHertz((sysclk.into() as Hertz).0 / 1_000);

    let ticks = period.to_ticks() * fcpu_khz.0 / in_freq_khz.0;

    Duration::ticks(ticks)
}

pub fn print_clock_config(clocks: &Option<stm32l4xx_hal::rcc::Clocks>, usb_state: &str) {
    if let Some(clocks) = clocks {
        //defmt::info!(
        //    "Clock config: CPU={}, pclk1={}, pclk2={}, USB: {}",
        //    clocks.sysclk().0,
        //    clocks.pclk1().0,
        //    clocks.pclk2().0,
        //    usb_state
        //);
    } else {
        //defmt::error!("System clock not configures yet");
    }
}
