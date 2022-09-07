use core::convert::Infallible;

use embedded_hal::{PwmPin, digital::v2::OutputPin};

pub struct Laser<PWM, ENABLE: OutputPin<Error = Infallible>> {
    laser_pwm_pin: PWM,
    laser_enable_pin: ENABLE,
}

impl<PWM: PwmPin<Duty = u16>, ENABLE: OutputPin<Error = Infallible>> Laser<PWM, ENABLE> {
    pub fn new(mut laser_pwm_pin: PWM, laser_enable_pin: ENABLE) -> Self {
        laser_pwm_pin.enable();

        let mut res = Self {
            laser_pwm_pin,
            laser_enable_pin,
        };

        res.set_power(0f64);

        res
    }

    pub(crate) fn enable(&mut self) {
        let _ = self
            .laser_enable_pin
            .set_state(crate::config::LASER_EN_ACTIVE_LVL.into());
    }

    pub(crate) fn disable(&mut self) {
        let _ = self
            .laser_enable_pin
            .set_state((!crate::config::LASER_EN_ACTIVE_LVL).into());
    }

    pub(crate) fn set_power(&mut self, power: f64) {
        let power = crate::support::map(
            power,
            0.0,
            crate::config::MOTION_MAX_S,
            0,
            self.laser_pwm_pin.get_max_duty(),
        );
        self.laser_pwm_pin.set_duty(power);
    }
}
