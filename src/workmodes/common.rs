use freertos_rust::{Duration, DurationTicks};
use stm32l4xx_hal::time::{Hertz, MegaHertz};

pub static HSE_FREQ: MegaHertz = MegaHertz(12);

pub fn calc_monitoring_period<D: DurationTicks, F: Into<Hertz>>(period: D, sysclk: F) -> Duration {
    let in_freq_khz: Hertz = crate::workmodes::common::HSE_FREQ.into();
    let fcpu_khz: Hertz = sysclk.into();

    let ticks = period.to_ticks() as u64 * fcpu_khz.0 as u64 / in_freq_khz.0 as u64;

    Duration::ticks(ticks as u32)
}

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

pub fn enable_dma_clocking() {
    use stm32l4xx_hal::stm32;

    // https://github.com/probe-rs/probe-rs/issues/350#issuecomment-740550519
    let rcc = unsafe { &*stm32::RCC::ptr() };
    rcc.ahb1enr.modify(|_, w| w.dma1en().set_bit());

    let etm = unsafe { &*stm32::DBGMCU::ptr() };
    etm.cr.modify(|_, w| {
        w.dbg_sleep()
            .set_bit()
            .dbg_standby()
            .set_bit()
            .dbg_stop()
            .set_bit()
    });
}
