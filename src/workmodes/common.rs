use defmt::{write, Format};
use freertos_rust::{Duration, DurationTicks, FreeRtosTickType, Task, TaskPriority};

use stm32l4xx_hal::time::{Hertz, MegaHertz};

use crate::threads;

pub static HSE_FREQ: MegaHertz = MegaHertz(12);
pub static MONITOR_MSG_PERIOD: u32 = 1000;

static mut TICKS_TO_S_DEVIDER: u32 = 1000;

#[derive(Default)]
pub struct SMs {
    second: u32,
    ms: u16,
}

pub trait HertzExt {
    fn duration_ms(&self, ms: u32) -> Duration;
}

pub trait FreeRtosTickTypeExt {
    fn to_hmss(&self) -> SMs;
}

impl HertzExt for Hertz {
    fn duration_ms(&self, ms: u32) -> Duration {
        to_real_period(Duration::ms(ms), self.clone())
    }
}

impl FreeRtosTickTypeExt for FreeRtosTickType {
    fn to_hmss(&self) -> SMs {
        SMs {
            ms: unsafe { (self % TICKS_TO_S_DEVIDER) as u16 },
            second: unsafe { self / TICKS_TO_S_DEVIDER },
        }
    }
}

impl Format for SMs {
    fn format(&self, fmt: defmt::Formatter) {
        write!(fmt, "{:07}.{:03}", self.second, self.ms);
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
            .priority(TaskPriority(1))
            .start(move |_| threads::monitor::monitord(monitoring_period))?;
    }

    unsafe {
        TICKS_TO_S_DEVIDER = to_real_period(Duration::ms(1000), sysclk).to_ticks();
    }

    Ok(())
}
