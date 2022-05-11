use alloc::sync::Arc;
use freertos_rust::{Duration, DurationTicks, Mutex};

use qspi_stm32lx3::{
    qspi::{ClkPin, IO0Pin, IO1Pin, IO2Pin, IO3Pin, NCSPin, Qspi},
    stm32l4x3,
};
use stm32l4xx_hal::{
    rcc::{PllConfig, PllDivider},
    time::Hertz,
};

use super::output_storage::OutputStorage;

pub fn to_pll_devider(v: u32) -> PllDivider {
    match v {
        2 => PllDivider::Div2,
        4 => PllDivider::Div4,
        6 => PllDivider::Div6,
        8 => PllDivider::Div8,
        _ => panic!(),
    }
}

pub trait ClockConfigProvider {
    fn core_frequency() -> Hertz;
    fn apb1_frequency() -> Hertz;
    fn apb2_frequency() -> Hertz;
    fn master_counter_frequency() -> Hertz;
    fn pll_config() -> PllConfig;
    fn xtal2master_freq_multiplier() -> f64;
}

#[derive(Default)]
pub struct Ticks(pub u32);

pub trait HertzExt {
    fn duration_ms(&self, ms: u32) -> Duration;

    fn real_duration_from_ticks(&self, ticks: u32) -> Duration;
}

impl HertzExt for Hertz {
    fn duration_ms(&self, ms: u32) -> Duration {
        to_real_period(Duration::ms(ms), self.clone())
    }

    fn real_duration_from_ticks(&self, ticks: u32) -> Duration {
        from_real_period(ticks, self.clone())
    }
}

impl defmt::Format for Ticks {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{:09}", self.0);
    }
}

pub fn to_real_period<D: DurationTicks, F: Into<Hertz>>(period: D, sysclk: F) -> Duration {
    let in_freq_hz = Hertz::Hz(crate::config::FREERTOS_CONFIG_FREQ);
    let fcpu_hz: Hertz = sysclk.into();

    let ticks = period.to_ticks() as u64 * fcpu_hz.to_Hz() as u64 / in_freq_hz.to_Hz() as u64;

    Duration::ticks(ticks as u32)
}

pub fn from_real_period<F: Into<Hertz>>(period: u32, sysclk: F) -> Duration {
    let in_freq_hz = Hertz::Hz(crate::config::FREERTOS_CONFIG_FREQ);
    let fcpu_hz: Hertz = sysclk.into();

    let ticks = period as u64 * in_freq_hz.to_Hz() as u64 / fcpu_hz.to_Hz() as u64;

    Duration::ticks(ticks as u32)
}

pub fn print_clock_config(clocks: &Option<stm32l4xx_hal::rcc::Clocks>, usb_state: &str) {
    if let Some(clocks) = clocks {
        defmt::info!(
            "Clock config: CPU={} Mhz, pclk1={} MHz, pclk2={}MHz, USB: {}",
            clocks.hclk().to_MHz(),
            clocks.pclk1().to_MHz(),
            clocks.pclk2().to_MHz(),
            usb_state
        );
    } else {
        defmt::error!("System clock not configures yet");
    }
}

pub fn create_monitor(
    _sysclk: Hertz,
    _output: Arc<Mutex<OutputStorage>>,
) -> Result<(), freertos_rust::FreeRtosError> {
    #[cfg(feature = "monitor")]
    #[cfg(debug_assertions)]
    {
        use crate::threads;
        use freertos_rust::{Task, TaskPriority};

        static MONITOR_STACK_SIZE: u16 = 840;
        pub static MONITOR_MSG_PERIOD: u32 = 1000;

        defmt::trace!("Creating monitor thread...");
        let monitoring_period = _sysclk.duration_ms(MONITOR_MSG_PERIOD);
        Task::new()
            .name("Monitord")
            .stack_size(MONITOR_STACK_SIZE)
            .priority(TaskPriority(crate::config::MONITOR_TASK_PRIO))
            .start(move |_| threads::monitor::monitord(monitoring_period, _output))?;
    }

    Ok(())
}

#[allow(unused)]
pub fn create_qspi<CLK, NCS, IO0, IO1, IO2, IO3, RESET>(
    pins: (CLK, NCS, IO0, IO1, IO2, IO3),
    mut reset: RESET,
    ahb3: &mut stm32l4xx_hal::rcc::AHB3,
) -> (Qspi<(CLK, NCS, IO0, IO1, IO2, IO3)>, RESET)
where
    CLK: ClkPin<stm32l4x3::QUADSPI>,
    NCS: NCSPin<stm32l4x3::QUADSPI>,
    IO0: IO0Pin<stm32l4x3::QUADSPI>,
    IO1: IO1Pin<stm32l4x3::QUADSPI>,
    IO2: IO2Pin<stm32l4x3::QUADSPI>,
    IO3: IO3Pin<stm32l4x3::QUADSPI>,
    RESET: stm32l4xx_hal::prelude::OutputPin,
{
    // reset flash
    let _ = reset.set_low();
    cortex_m::asm::delay(1);
    let _ = reset.set_high();

    (
        Qspi::new(
            unsafe { stm32l4x3::QUADSPI::new() },
            pins,
            unsafe { core::mem::transmute(ahb3) },
            qspi_stm32lx3::qspi::QspiConfig::default(),
        ),
        reset,
    )
}
