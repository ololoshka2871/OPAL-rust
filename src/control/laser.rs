use core::convert::Infallible;

use embedded_hal::digital::v2::OutputPin;
use num_derive::FromPrimitive;

use crate::support::{
    parallel_input_bus::ParallelInputBus, parallel_output_bus::ParallelOutputBus,
};

/// Table 5 Definition of alarm status.
#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum LaserStatus {
    TemperatureAlarm = 0,
    Normal = 1,
    SystemAlarm = 3,
    SupplyVoltageAlarm = 4,
}

pub trait LaserInterface {
    /// устанавливает Power Setting
    /// Включает меандр на Sync
    /// Включает laser_emission_enable
    /// laser_emission_modulation duty = 0
    fn enable(&mut self);

    /// laser_emission_modulation duty = 0
    /// Выключает laser_emission_enable
    /// Выключает меандр на Sync
    /// устанавливает Power Setting = 0
    fn disable(&mut self);

    /// Устанавливает laser_emission_modulation 0 - 100
    fn set_power_pwm(&mut self, power: f32);

    /// Устанваливает Power Setting
    fn set_pump_power(&mut self, power_code: u8);

    /// прочитать статус лазера
    fn get_status(&self) -> LaserStatus;

    /// установить мощность красного лазера
    fn set_red_laser_power(&mut self, power: f32);
}

pub struct Laser<PBUS, ABUS, OUTPIN, EM, EE, ES, RL>
where
    PBUS: ParallelOutputBus<Output = u8>,
    ABUS: ParallelInputBus<Input = u8>,
    OUTPIN: OutputPin<Error = Infallible>,
{
    power_set_bus: PBUS,
    power_latch_pin: Option<OUTPIN>,

    alarm_bus: ABUS,

    laser_emission_modulation: EM,
    laser_emission_enable: EE,
    laser_sync: ES,

    laser_red_beam: RL,

    current_power_seting: u8,
    current_em_mod_seting: u16,

    enabled: bool,
}

pub mod laser_pa0_7_pa13_15_tom4_tim1;
