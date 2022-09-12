use core::convert::Infallible;

use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{timer::PwmChannel, device::{TIM4, TIM1}};

use crate::support::{parallel_input_bus, parallel_output_bus::ParallelOutputBus};

impl<PBUS, ABUS, OUTPIN, EM, EE, ES, RL> super::Laser<PBUS, ABUS, OUTPIN, EM, EE, ES, RL>
where
    PBUS: ParallelOutputBus<Output = u8>,
    ABUS: parallel_input_bus::ParallelInputBus<Input = u8>,
    OUTPIN: OutputPin<Error = Infallible>,
{
    pub fn new(
        power_set_bus: PBUS,
        power_latch_pin: OUTPIN,
        alarm_bus: ABUS,

        laser_emission_modulation: EM,
        laser_emission_enable: EE,
        laser_sync: ES,

        laser_red_beam: RL,
    ) -> Self {
        Self {
            power_set_bus,
            power_latch_pin,
            alarm_bus,
            laser_emission_modulation,
            laser_emission_enable,
            laser_sync,
            laser_red_beam,
        }
    }
}

impl<PBUS, ABUS, OUTPIN> super::LaserInterface
    for super::Laser<
        PBUS,
        ABUS,
        OUTPIN,
        PwmChannel<TIM4, 2>,
        PwmChannel<TIM4, 3>,
        PwmChannel<TIM4, 1>,
        PwmChannel<TIM1, 2>,
    >
where
    PBUS: ParallelOutputBus<Output = u8>,
    ABUS: parallel_input_bus::ParallelInputBus<Input = u8>,
    OUTPIN: OutputPin<Error = Infallible>,
{
    fn enable(&mut self) {
        todo!()
    }

    fn disable(&mut self) {
        todo!()
    }

    fn set_power(&mut self, power: f64) {
        todo!()
    }
}
