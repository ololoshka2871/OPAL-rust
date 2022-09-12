use core::{arch::asm, convert::Infallible};

use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{
    device::{TIM1, TIM4},
    timer::PwmChannel,
};

use crate::support::{parallel_input_bus, parallel_output_bus::ParallelOutputBus};

impl<PBUS, ABUS, OUTPIN, EM, EE, ES, RL> super::Laser<PBUS, ABUS, OUTPIN, EM, EE, ES, RL>
where
    PBUS: ParallelOutputBus<Output = u8>,
    ABUS: parallel_input_bus::ParallelInputBus<Input = u8>,
    OUTPIN: OutputPin<Error = Infallible>,
{
    pub fn new(
        power_set_bus: PBUS,
        power_latch_pin: Option<OUTPIN>,
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

            current_power_seting: 0,
            current_em_mod_seting: 0,

            enabled: false,
        }
    }

    fn power2_pwm(power: f32, max_duty: u16) -> u16 {
        (max_duty as f32 / 100.0 * power) as u16
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
        if !self.enabled {
            self.set_pump_power(self.current_power_seting);

            self.laser_sync.set_duty(self.laser_sync.get_max_duty() / 2);
            self.laser_sync.enable();

            self.laser_emission_enable
                .set_duty(self.laser_emission_enable.get_max_duty());
            self.laser_emission_modulation.enable();

            self.laser_emission_modulation
                .set_duty(self.current_em_mod_seting);
            self.laser_emission_modulation.enable();
            self.enabled = true;
        }
    }

    fn disable(&mut self) {
        self.set_power_pwm(0.0);

        self.laser_emission_modulation.set_duty(0);
        self.laser_emission_modulation.disable();

        self.laser_emission_enable.set_duty(0);
        self.laser_emission_enable.disable();

        self.laser_sync.set_duty(0);
        self.laser_sync.disable();

        self.set_pump_power(0);

        self.enabled = false;
    }

    fn set_power_pwm(&mut self, mut power: f32) {
        if power > 100.0 {
            power = 100.0
        }

        self.current_em_mod_seting =
            Self::power2_pwm(power, self.laser_emission_modulation.get_max_duty());
        self.laser_emission_modulation
            .set_duty(self.current_em_mod_seting);
    }

    fn set_pump_power(&mut self, power_code: u8) {
        self.power_set_bus.set(power_code);
        if let Some(latch) = &mut self.power_latch_pin {
            let _ = latch.set_high();
            for _ in 0..1000 {
                unsafe { asm!("nop") };
            }
            let _ = latch.set_low();
        }
    }

    fn get_status(&self) -> super::LaserStatus {
        num::FromPrimitive::from_u8(self.alarm_bus.get()).unwrap_or(super::LaserStatus::SystemAlarm)
    }

    fn set_red_laser_power(&mut self, power: f32) {
        if power > 0.0 {
            self.laser_red_beam
                .set_duty(Self::power2_pwm(power, self.laser_red_beam.get_max_duty()));
            self.laser_red_beam.enable();
        } else {
            self.laser_red_beam.set_duty(0);
            self.laser_red_beam.disable();
        }
    }
}
