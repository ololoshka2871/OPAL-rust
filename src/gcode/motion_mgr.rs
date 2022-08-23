use core::convert::Infallible;
use core::fmt::Display;

use alloc::{fmt::format, string::String};
use embedded_hal::PwmPin;
use stm32l4xx_hal::prelude::OutputPin;

use crate::config;

use crate::control::laser::Laser;
use crate::control::xy2_100::XY2_100;
use crate::time_base::master_counter::MasterTimerInfo;

use super::GCode;

#[derive(PartialEq, Clone, Copy)]
pub enum MotionStatus {
    IDLE,
    INTERPOLATING,
}

pub struct MotionMGR<PWM, LASEREN, GALVOEN>
where
    PWM: PwmPin<Duty = u16>,
    GALVOEN: OutputPin<Error = Infallible>,
    LASEREN: OutputPin<Error = Infallible>,
{
    _status: MotionStatus,
    is_move_first_interpolation: bool,

    current_startnanos: u64,
    current_endnanos: u64,

    _now: u64,
    current_code: u32,

    current_from_x: f64,
    current_from_y: f64,
    current_from_z: f64,
    current_distance_x: f64,
    current_distance_y: f64,
    _current_distance_z: f64,
    current_to_x: f64,
    current_to_y: f64,
    current_to_z: f64,
    current_cmd_x: f64,
    current_cmd_y: f64,
    current_cmd_z: f64,
    _current_i: f64,
    _current_j: f64,
    current_f: f64,
    current_s: f64,
    current_duration: f64,
    current_absolute: bool,
    current_laserenabled: bool,
    laser_changed: bool,

    laser: Laser<PWM, LASEREN>,
    galvo: XY2_100<GALVOEN>,

    master: MasterTimerInfo,
    to_nanos: f64,
}

impl<PWM, LASEREN, GALVOEN> MotionMGR<PWM, LASEREN, GALVOEN>
where
    PWM: PwmPin<Duty = u16>,
    GALVOEN: OutputPin<Error = Infallible>,
    LASEREN: OutputPin<Error = Infallible>,
{
    pub fn new(
        laser: Laser<PWM, LASEREN>,
        galvo: XY2_100<GALVOEN>,
        master: MasterTimerInfo,
        to_nanos: f64,
    ) -> Self {
        Self {
            _status: MotionStatus::IDLE,
            is_move_first_interpolation: false,
            current_startnanos: 0,
            current_endnanos: 0,
            _now: 0,
            current_code: 0,
            current_from_x: 0f64,
            current_from_y: 0f64,
            current_from_z: 0f64,
            current_distance_x: 0f64,
            current_distance_y: 0f64,
            _current_distance_z: 0f64,
            current_to_x: 0f64,
            current_to_y: 0f64,
            current_to_z: 0f64,
            current_cmd_x: 0f64,
            current_cmd_y: 0f64,
            current_cmd_z: 0f64,
            _current_i: 0f64,
            _current_j: 0f64,
            current_f: 0f64,
            current_s: 0f64,
            current_duration: 0f64,
            current_absolute: false,
            current_laserenabled: false,
            laser_changed: false,

            laser,
            galvo,

            master,
            to_nanos,
        }
    }

    pub fn begin(&mut self) {
        self.set_galvo_position(0.0, 0.0);
    }

    pub fn process(&mut self, gcode: GCode) -> Result<(), String> {
        if self._status == MotionStatus::IDLE {
            self.process_gcodes(gcode)
        } else {
            Err("Motion busy!".into())
        }
    }

    pub fn tic(&mut self) -> MotionStatus {
        self._now = self.nanos();
        if self._status == MotionStatus::INTERPOLATING {
            if self.interpolate_move() {
                self.set_galvo_position(self.current_cmd_x, self.current_cmd_y);
            }
        }

        if self.laser_changed {
            if self.current_laserenabled {
                self.set_laser_power(self.current_s);
                self.laser_changed = false;
            } else {
                self.set_laser_power(0f64);
            }
            self.laser_changed = false;
        }

        self._status
    }

    fn process_gcodes(&mut self, gcode: GCode) -> Result<(), String> {
        if gcode.is_g_code() && [0, 1, 2, 3, 28, 90, 91].contains(&gcode.code()) {
            // Valid G-codes
            match gcode.code() {
                0 => {
                    self.current_code = 0;
                    self.set_xy(&gcode)?;
                }
                1 => {
                    self.current_code = 1;
                    Self::set_value(
                        &mut self.current_f,
                        gcode.get_f(),
                        'F',
                        i32::MAX as f64,
                        0f64,
                    )?;
                    {
                        let mut new_s = 0f64;
                        if let Ok(()) = Self::set_value(
                            &mut new_s,
                            gcode.get_s(),
                            'S',
                            config::MOTION_MAX_S,
                            -0f64,
                        ) {
                            self.laser_changed = new_s != self.current_s;
                            self.current_s = new_s;
                        }
                    }

                    self.set_xy(&gcode)?;
                }
                28 => {
                    self.current_code = 28;
                    self.current_to_x = 0f64;
                    self.current_to_y = 0f64;
                    self.current_to_z = 0f64;
                }
                _ => return Ok(()),
            }
            self._status = MotionStatus::INTERPOLATING;
        } else {
            // possibly M-Code or prev code continue
            self.process_mcode(gcode)?
        }
        Ok(())
    }

    fn set_xy(&mut self, gcode: &GCode) -> Result<(), String> {
        if self.current_absolute {
            Self::set_value(
                &mut self.current_to_x,
                gcode.get_x(),
                'X',
                config::MOTION_X_RANGE / 2.0,
                -config::MOTION_X_RANGE / 2.0,
            )?;
            Self::set_value(
                &mut self.current_to_y,
                gcode.get_y(),
                'Y',
                config::MOTION_Y_RANGE / 2.0,
                -config::MOTION_Y_RANGE / 2.0,
            )?;

            Self::set_value(
                &mut self.current_to_z,
                gcode.get_z(),
                'Z',
                config::MOTION_Z_RANGE / 2.0,
                -config::MOTION_Z_RANGE / 2.0,
            )?;
        } else {
            Self::set_value_g91(
                &mut self.current_to_x,
                self.current_from_x,
                gcode.get_x(),
                'X',
                config::MOTION_X_RANGE / 2.0,
                -config::MOTION_X_RANGE / 2.0,
            )?;
            Self::set_value_g91(
                &mut self.current_to_y,
                self.current_from_y,
                gcode.get_y(),
                'Y',
                config::MOTION_Y_RANGE / 2.0,
                -config::MOTION_Y_RANGE / 2.0,
            )?;

            Self::set_value_g91(
                &mut self.current_to_z,
                self.current_from_z,
                gcode.get_z(),
                'Z',
                config::MOTION_Z_RANGE / 2.0,
                -config::MOTION_Z_RANGE / 2.0,
            )?;
        }
        Ok(())
    }

    fn set_value<T: Copy + core::cmp::PartialOrd + Display>(
        dest: &mut T,
        src: T,
        name: char,
        plimit: T,
        nlimit: T,
    ) -> Result<(), String> {
        if src > plimit {
            Err(format(format_args!("{} above limit {}", name, plimit)))
        } else if src < nlimit {
            Err(format(format_args!("{} below limit {}", name, nlimit)))
        } else {
            *dest = src;
            Ok(())
        }
    }

    fn set_value_g91<T: Copy + core::cmp::PartialOrd + core::ops::Add<Output = T> + Display>(
        dest: &mut T,
        current: T,
        src: T,
        name: char,
        plimit: T,
        nlimit: T,
    ) -> Result<(), String> {
        let to = current + src;
        Self::set_value(dest, to, name, plimit, nlimit)
    }

    fn process_mcode(&mut self, gcode: GCode) -> Result<(), String> {
        if gcode.is_m_code() {
            match gcode.code() {
                3 | 4 => {
                    let new_s = gcode.get_s();

                    Self::set_value(&mut self.current_s, new_s, 'S', config::MOTION_MAX_S, -0f64)?;
                    self.current_laserenabled = true;
                    self.laser_changed = true;
                }

                5 => {
                    self.current_laserenabled = false;
                    self.laser_changed = true;
                }

                8 => { /* Coolant on */ }
                9 => { /* Coolant off */ }

                17 => self.galvo.enable(),
                18 => self.galvo.disable(),

                80 => self.laser.enable(),
                81 => self.laser.disable(),

                _ => {}
            }
            Ok(())
        } else {
            /* TODO */
            Ok(())
        }
    }

    fn interpolate_move(&mut self) -> bool {
        if self.is_move_first_interpolation {
            if [0, 28].contains(&self.current_code) {
                // don't interpolate
                self.current_from_x = self.current_to_x;
                self.current_from_y = self.current_to_y;
                self.current_from_z = self.current_to_z;
                self.current_cmd_x = self.current_to_x;
                self.current_cmd_y = self.current_to_y;
                self.current_cmd_z = self.current_to_z;
                self._status = MotionStatus::IDLE;
                self.is_move_first_interpolation = true;
                return true;
            }
            if self.current_code == 1 {
                // G1
                self.current_distance_x = self.current_to_x - self.current_from_x;
                self.current_distance_y = self.current_to_y - self.current_from_y;
                calculate_move_length_nanos(
                    self.current_distance_x,
                    self.current_distance_y,
                    self.current_f,
                    &mut self.current_duration,
                );
                self.current_startnanos = self._now;
                self.current_endnanos = self._now + libm::round(self.current_duration) as u64;
                self.is_move_first_interpolation = false;
            }
        }

        //Actual interpolation
        if self._now >= self.current_endnanos {
            //done interpolating
            self.current_from_x = self.current_to_x;
            self.current_from_y = self.current_to_y;
            self.current_from_z = self.current_to_z;
            self.current_cmd_x = self.current_to_x;
            self.current_cmd_y = self.current_to_y;
            self.current_cmd_z = self.current_to_z;
            self._status = MotionStatus::IDLE;
            self.is_move_first_interpolation = true;
            return self._now == self.current_endnanos;
        } else {
            let fraction_of_move =
                (self._now - self.current_startnanos) as f64 / self.current_duration;
            self.current_cmd_x = self.current_from_x + (self.current_distance_x * fraction_of_move);
            self.current_cmd_y = self.current_from_y + (self.current_distance_y * fraction_of_move);
            return true;
        }
    }

    fn set_galvo_position(&mut self, x: f64, y: f64) {
        use crate::support::map;

        let cmd_x = if config::AXIS_INVERSE_X {
            map(
                x,
                -config::MOTION_X_RANGE / 2.0,
                config::MOTION_X_RANGE / 2.0,
                u16::MAX,
                u16::MIN,
            )
        } else {
            map(
                x,
                -config::MOTION_X_RANGE / 2.0,
                config::MOTION_X_RANGE / 2.0,
                u16::MIN,
                u16::MAX,
            )
        };

        let cmd_y = if config::AXIS_INVERSE_Y {
            map(
                y,
                -config::MOTION_Y_RANGE / 2.0,
                config::MOTION_Y_RANGE / 2.0,
                u16::MAX,
                u16::MIN,
            )
        } else {
            map(
                y,
                -config::MOTION_Y_RANGE / 2.0,
                config::MOTION_Y_RANGE / 2.0,
                u16::MIN,
                u16::MAX,
            )
        };

        self.galvo.set_pos(cmd_x, cmd_y);
    }

    fn nanos(&self) -> u64 {
        (self.master.value64().0 as f64 * self.to_nanos) as u64
    }

    fn set_laser_power(&mut self, power: f64) {
        self.laser.set_power(power);
        //defmt::trace!("Laser power: {}%", power);
    }
}

fn calculate_move_length_nanos(xdist: f64, ydist: f64, move_velocity: f64, result: &mut f64) {
    let length_of_move = libm::sqrt(xdist * xdist + ydist * ydist);
    *result = length_of_move * 1000.0 * 1000.0 * 1000.0 / move_velocity;
}
