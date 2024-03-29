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

        laser_tim_freq: systick_monotonic::fugit::Hertz<u32>,
    ) -> Self {
        Self {
            power_set_bus,
            power_latch_pin,
            alarm_bus,
            laser_emission_modulation,
            laser_emission_enable,
            laser_sync,
            laser_red_beam,

            laser_tim_freq,

            current_power_seting: 0,
            power: 0.0,

            frequency: crate::config::LASER_SYNC_CLOCK_KHZ,

            enabled: false,
        }
    }

    fn power2_pwm(power: f32, max_duty: u16) -> u16 {
        (max_duty as f32 / 100.0 * power) as u16
    }

    fn impl_set_pump_power(&mut self, power_code: u8) {
        self.power_set_bus.set(power_code);
        if let Some(latch) = &mut self.power_latch_pin {
            let _ = latch.set_high();
            for _ in 0..100 {
                unsafe { asm!("nop") };
            }
            let _ = latch.set_low();
        }
    }

    fn impl_set_frequency(&mut self) {
        use stm32f1xx_hal::pac;

        const fn compute_arr_presc(freq: u32, clock: u32) -> (u32, u32) {
            let ticks = clock / freq;
            let psc = (ticks - 1) / (1 << 16);
            let arr = ticks / (psc + 1) - 1;
            (psc, arr)
        }

        let (psc, arr) = compute_arr_presc(self.frequency, self.laser_tim_freq.raw());

        unsafe {
            (*pac::TIM4::ptr()).psc.write(|w| w.bits(psc));
            (*pac::TIM4::ptr()).arr.write(|w| w.bits(arr));
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
        if !self.enabled {
            self.impl_set_pump_power(self.current_power_seting);

            self.impl_set_frequency();

            self.laser_sync.set_duty(self.laser_sync.get_max_duty() / 2);
            self.laser_sync.enable();

            self.laser_emission_enable
                .set_duty(self.laser_emission_enable.get_max_duty());
            self.laser_emission_enable.enable();

            let current_em_mod_seting =
                Self::power2_pwm(self.power, self.laser_emission_modulation.get_max_duty());
            self.laser_emission_modulation
                .set_duty(current_em_mod_seting);

            self.laser_emission_modulation.enable();
            self.enabled = true;
        }
    }

    fn disable(&mut self) {
        self.laser_emission_modulation.set_duty(0);
        self.laser_emission_modulation.disable();

        self.laser_emission_enable.set_duty(0);
        self.laser_emission_enable.disable();

        self.laser_sync.set_duty(0);
        self.laser_sync.disable();

        self.impl_set_pump_power(0);

        self.enabled = false;
    }

    fn set_power_pwm(&mut self, mut power: f32) {
        if power > 100.0 {
            power = 100.0
        }

        if power < 0.0 {
            power = 0.0
        }

        self.power = power;
    }

    fn set_pump_power(&mut self, power_code: u8) {
        self.current_power_seting = power_code;
    }

    fn set_frequency(&mut self, frequency: u32) {
        self.frequency = frequency;
    }

    fn get_status(&self) -> super::LaserStatus {
        match self.alarm_bus.get() {
            0 => super::LaserStatus::TemperatureAlarm,
            1 => super::LaserStatus::Normal,
            3 => super::LaserStatus::SystemAlarm,
            4 => super::LaserStatus::SupplyVoltageAlarm,
            _ => super::LaserStatus::SystemAlarm,
        }
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

    fn debug_set_ee(&mut self, enable: bool) {
        let _ = self.laser_emission_enable.set_duty(if enable {
            self.laser_emission_enable.get_max_duty()
        } else {
            0
        });
    }
}
