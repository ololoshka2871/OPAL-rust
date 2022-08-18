use core::fmt::Display;

use alloc::{fmt::format, string::String};

use crate::config;

use crate::control::xy2_100::xy2_100;

use super::GCode;

#[derive(PartialEq, Clone, Copy)]
pub enum MotionStatus {
    IDLE,
    INTERPOLATING,
}

pub struct MotionMGR {
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

    _laser: u32,
    galvo: xy2_100,
}

impl MotionMGR {
    pub fn new(laser: u32, galvo: xy2_100) -> Self {
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

            _laser: laser,
            galvo,
        }
    }

    pub fn process(&mut self, gcode: GCode) -> Result<(), String> {
        if self._status == MotionStatus::IDLE {
            self.process_gcodes(gcode)
        } else {
            Err("Motion busy!".into())
        }
    }

    pub fn tic(&mut self) -> MotionStatus {
        self._now = nanos();
        if self._status == MotionStatus::INTERPOLATING {
            self.interpolate_move();
        }

        self.set_galvo_position(self.current_cmd_x, self.current_cmd_y);

        if self.current_laserenabled {
            if self.laser_changed {
                self.set_laser_power(self.current_s);
                self.laser_changed = false;
            }
        } else {
            self.set_laser_power(0f64);
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

                5 => self.current_laserenabled = false,

                9 => {} /* setNextFWDMSG ??? */

                17 => { /* TODO enable GALVO */ }
                18 => { /* TODO disable GALVO */ }

                80 => { /* TODO enable LASER */ }
                81 => { /* TODO disable LASER */ }
                _ => {}
            }
            Ok(())
        } else {
            /* TODO */
            Ok(())
        }
    }

    fn interpolate_move(&mut self) {
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
                return;
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
            return;
        } else {
            let fraction_of_move =
                (self._now - self.current_startnanos) as f64 / self.current_duration;
            self.current_cmd_x = self.current_from_x + (self.current_distance_x * fraction_of_move);
            self.current_cmd_y = self.current_from_y + (self.current_distance_y * fraction_of_move);
            return;
        }
    }

    fn set_galvo_position(&mut self, x: f64, y: f64) {
        let cmd_x = if config::AXIS_INVERSE_X {
            map(
                x,
                -config::MOTION_X_RANGE / 2.0,
                config::MOTION_X_RANGE / 2.0,
                u16::MIN,
                u16::MAX,
            )
        } else {
            map(
                x,
                -config::MOTION_X_RANGE / 2.0,
                config::MOTION_X_RANGE / 2.0,
                u16::MAX,
                u16::MIN,
            )
        };

        let cmd_y = if config::AXIS_INVERSE_Y {
            map(
                y,
                -config::MOTION_Y_RANGE / 2.0,
                config::MOTION_Y_RANGE / 2.0,
                u16::MIN,
                u16::MAX,
            )
        } else {
            map(
                y,
                -config::MOTION_Y_RANGE / 2.0,
                config::MOTION_Y_RANGE / 2.0,
                u16::MAX,
                u16::MIN,
            )
        };

        self.galvo.set_pos(cmd_x, cmd_y);
    }

    fn set_laser_power(&mut self, _power: f64) {}
}

fn nanos() -> u64 {
    0
}

fn calculate_move_length_nanos(xdist: f64, ydist: f64, move_velocity: f64, result: &mut f64) {
    let length_of_move = libm::sqrt(xdist * xdist + ydist * ydist);
    *result = length_of_move * 1000.0 * 1000.0 * 1000.0 / move_velocity;
}

/// Маппинг диопазонов
/// *-------х-------*
/// ^min    ^v      ^max
/// percent = (v - min) / (max - min)
///
/// *-------x-------*
/// ^left   ^res    ^right
/// res = left + (right - left) * percent
fn map(v: f64, min: f64, max: f64, left: u16, right: u16) -> u16 {
    let percent = (max - min) / (v - min);
    let _left = left as i32;
    let right = right as i32;
    left + (((right - _left) as f64 * percent) as u16)
}
