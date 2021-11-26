use defmt::{write, Format};
use freertos_rust::{Duration, DurationTicks, Task, TaskPriority};

use stm32l4xx_hal::time::{Hertz, MegaHertz};

use crate::threads;

pub static HSE_FREQ: MegaHertz = MegaHertz(12);
pub static MONITOR_MSG_PERIOD: u32 = 1000;

#[derive(Default)]
pub struct Ticks(pub u32);

pub trait HertzExt {
    fn duration_ms(&self, ms: u32) -> Duration;
}

impl HertzExt for Hertz {
    fn duration_ms(&self, ms: u32) -> Duration {
        to_real_period(Duration::ms(ms), self.clone())
    }
}

impl Format for Ticks {
    fn format(&self, fmt: defmt::Formatter) {
        write!(fmt, "{:09}", self.0);
    }
}

pub fn to_real_period<D: DurationTicks, F: Into<Hertz>>(period: D, sysclk: F) -> Duration {
    let in_freq_hz: Hertz = HSE_FREQ.into();
    let fcpu_hz: Hertz = sysclk.into();

    let ticks = period.to_ticks() as u64 * fcpu_hz.0 as u64 / in_freq_hz.0 as u64;

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

pub fn create_monitor(sysclk: Hertz) -> Result<(), freertos_rust::FreeRtosError> {
    static MONITOR_STACK_SIZE: u16 = 384;

    #[cfg(debug_assertions)]
    {
        defmt::trace!("Creating monitor thread...");
        let monitoring_period = sysclk.duration_ms(MONITOR_MSG_PERIOD);
        Task::new()
            .name("Monitord")
            .stack_size(MONITOR_STACK_SIZE)
            .priority(TaskPriority(crate::config::MONITOR_TASK_PRIO))
            .start(move |_| threads::monitor::monitord(monitoring_period))?;
    }

    Ok(())
}
