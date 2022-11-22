use core::fmt::Display;

use alloc::format;
use alloc::string::String;

use crate::{
    config, support::format_float_simple::format_float_simple,
    time_base::master_counter::MasterTimerInfo,
};

use super::GCode;

#[derive(PartialEq, Clone, Copy)]
pub enum MotionStatus {
    IDLE,
    INTERPOLATING,
}

pub struct MotionMGR<LASER, GALVO>
where
    GALVO: crate::control::xy2_100::XY2_100Interface,
    LASER: crate::control::laser::LaserInterface,
{
    _status: MotionStatus,
    is_move_first_interpolation: bool,

    current_startnanos: u64,
    current_endnanos: u64,

    _now: u64,
    current_code: u32,

    current_from_x: f32,
    current_from_y: f32,
    current_distance_x: f32,
    current_distance_y: f32,
    current_to_x: f32,
    current_to_y: f32,
    current_cmd_x: f32,
    current_cmd_y: f32,
    current_f: f32,
    current_s: f32,
    current_a: u8,
    current_duration: f32,
    current_absolute: bool,
    current_laserenabled: bool,
    current_red_laserenabled: bool,
    laser_changed: bool,

    laser: LASER,
    galvo: GALVO,

    master: MasterTimerInfo,
    to_nanos: f32,
}

impl<LASER, GALVO> MotionMGR<LASER, GALVO>
where
    GALVO: crate::control::xy2_100::XY2_100Interface,
    LASER: crate::control::laser::LaserInterface,
{
    pub fn new(galvo: GALVO, laser: LASER, master: MasterTimerInfo, to_nanos: f32) -> Self {
        Self {
            _status: MotionStatus::IDLE,
            is_move_first_interpolation: true,
            current_startnanos: 0,
            current_endnanos: 0,
            _now: 0,
            current_code: 0,
            current_from_x: 0f32,
            current_from_y: 0f32,
            current_distance_x: 0f32,
            current_distance_y: 0f32,
            current_to_x: 0f32,
            current_to_y: 0f32,
            current_cmd_x: 0f32,
            current_cmd_y: 0f32,
            current_f: 100f32,
            current_s: 0f32,
            current_a: 0,
            current_duration: 0f32,
            current_absolute: true,
            current_laserenabled: false,
            current_red_laserenabled: false,
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

    pub fn process(&mut self, gcode: &GCode) -> Result<Option<String>, String> {
        if self._status == MotionStatus::IDLE {
            use super::gcode::Code;
            match gcode.code() {
                Code::G(code) => self.process_gcodes(code, gcode),
                Code::M(code) => self.process_mcodes(code, gcode),
                Code::Empty => self.process_other(gcode),
            }?;
            Ok(None)
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
                self.laser.set_pump_power(self.current_a);
                self.laser.set_power_pwm(self.current_s);
                self.laser.enable();
            } else {
                self.laser.disable()
            }

            if self.current_red_laserenabled {
                self.laser.set_red_laser_power(self.current_s)
            } else {
                self.laser.set_red_laser_power(0.0);
            }

            self.laser_changed = false;
        }

        self._status
    }

    fn process_gcodes(&mut self, code: u32, gcode: &GCode) -> Result<(), String> {
        match code {
            0 => {
                self.current_code = 0;
                self.set_xya(&gcode)?;
            }
            1 => {
                self.current_code = 1;

                if let Some(new_s) = gcode.get_s() {
                    if let Err(_) = Self::set_value(
                        &mut self.current_s,
                        new_s,
                        'S',
                        config::MOTION_MAX_S,
                        -01f32,
                    ) {
                        if new_s > config::MOTION_MAX_S {
                            self.current_s = config::MOTION_MAX_S;
                        } else {
                            self.current_s = 0.0;
                        }
                    }
                }
                if let Some(new_f) = gcode.get_f() {
                    Self::set_value(&mut self.current_f, new_f, 'F', i32::MAX as f32, 0.01f32)?;
                }

                self.set_xya(&gcode)?;
            }
            28 => {
                self.current_code = 28;
                self.current_to_x = 0f32;
                self.current_to_y = 0f32;
            }
            90 => self.current_absolute = true,
            91 => self.current_absolute = false,

            _ => return Ok(()),
        }

        self._status = MotionStatus::INTERPOLATING;

        Ok(())
    }

    fn set_xya(&mut self, gcode: &GCode) -> Result<(), String> {
        if self.current_absolute {
            if let Some(to_x) = gcode.get_x() {
                Self::set_value(
                    &mut self.current_to_x,
                    to_x,
                    'X',
                    config::MOTION_X_RANGE / 2.0,
                    -config::MOTION_X_RANGE / 2.0,
                )?;
            }

            if let Some(to_y) = gcode.get_y() {
                Self::set_value(
                    &mut self.current_to_y,
                    to_y,
                    'Y',
                    config::MOTION_Y_RANGE / 2.0,
                    -config::MOTION_Y_RANGE / 2.0,
                )?;
            }
        } else {
            if let Some(to_x) = gcode.get_x() {
                Self::set_value_g91(
                    &mut self.current_to_x,
                    self.current_from_x,
                    to_x,
                    'X',
                    config::MOTION_X_RANGE / 2.0,
                    -config::MOTION_X_RANGE / 2.0,
                )?;
            }

            if let Some(to_y) = gcode.get_y() {
                Self::set_value_g91(
                    &mut self.current_to_y,
                    self.current_from_y,
                    to_y,
                    'Y',
                    config::MOTION_Y_RANGE / 2.0,
                    -config::MOTION_Y_RANGE / 2.0,
                )?;
            }
        }

        if let Some(new_a) = gcode.get_a() {
            if let Err(_) = Self::set_value(&mut self.current_a, new_a as u8, 'A', 0xff, 0) {
                if new_a > 255.0 {
                    self.current_a = 0xff;
                } else {
                    self.current_a = 0;
                }
            }
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
            Err(format!("{} above limit", name))
        } else if src < nlimit {
            Err(format!("{} below limit", name))
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

    fn process_mcodes(&mut self, code: u32, gcode: &GCode) -> Result<(), String> {
        match code {
            3 | 4 => {
                if let Some(new_s) = gcode.get_s() {
                    if let Err(_) =
                        Self::set_value(&mut self.current_s, new_s, 'S', config::MOTION_MAX_S, 0.0)
                    {
                        if new_s > config::MOTION_MAX_S {
                            self.current_s = config::MOTION_MAX_S;
                        } else {
                            self.current_s = 0.0;
                        }
                    }
                }

                if code == 3 {
                    self.current_laserenabled = true;
                } else {
                    self.current_red_laserenabled = true;
                }
                self.laser_changed = true;
            }

            5 => {
                self.current_laserenabled = false;
                self.current_red_laserenabled = false;
                self.laser_changed = true;
            }

            _ => {}
        }
        Ok(())
    }

    fn process_other(&mut self, gcode: &GCode) -> Result<(), String> {
        match self.current_code {
            0 | 1 => self.process_gcodes(self.current_code, gcode),
            c => Err(format!("Cannot continue move for G{}", c)),
        }
    }

    pub fn process_status_req(&self, req: &super::Request) -> Result<Option<String>, String> {
        match req {
            super::Request::Dollar('G') => Ok(Some(format!(
                // https://github.com/gnea/grbl/blob/master/doc/markdown/commands.md#g---view-gcode-parser-state
                "[GC:G{} G54 G17 G9{} G91.1 G94 G21 G40 G49 M0 M5 M9 T0 S{} F{}]\n\rok\n\r",
                self.current_code,
                (!self.current_absolute as u32),
                format_float_simple(self.current_s, 1),
                format_float_simple(self.current_f, 3),
            ))),
            super::Request::Status => {
                // re.compile(r"^<(\w*?),
                // MPos:([+\-]?\d*\.\d*),([+\-]?\d*\.\d*),([+\-]?\d*\.\d*)(?:,[+\-]?\d*\.\d*)?(?:,[+\-]?\d*\.\d*)?(?:,[+\-]?\d*\.\d*)?,
                // WPos:([+\-]?\d*\.\d*),([+\-]?\d*\.\d*),([+\-]?\d*\.\d*)(?:,[+\-]?\d*\.\d*)?(?:,[+\-]?\d*\.\d*)?(?:,[+\-]?\d*\.\d*)?(?:,.*)?>$")
                Ok(Some(format!(
                    "<Idle,MPos:{x:.5},{y:.5},0.0,WPos:{x:.5},{y:.5},0.0>\n\r",
                    x = format_float_simple(self.current_cmd_x, 5),
                    y = format_float_simple(self.current_cmd_y, 5),
                )))
            }
            super::Request::Dollar(dl) => Err(format!("Unsupported command ${}", dl)),
        }
    }

    fn interpolate_move(&mut self) -> bool {
        if self.is_move_first_interpolation {
            if [0, 28].contains(&self.current_code) {
                // don't interpolate
                self.current_from_x = self.current_to_x;
                self.current_from_y = self.current_to_y;
                self.current_cmd_x = self.current_to_x;
                self.current_cmd_y = self.current_to_y;
                self._status = MotionStatus::IDLE;
                self.is_move_first_interpolation = true;
                return true;
            }
            if self.current_code == 1 {
                // G1
                self.current_distance_x = self.current_to_x - self.current_from_x;
                self.current_distance_y = self.current_to_y - self.current_from_y;

                self.current_duration = calculate_move_length_nanos(
                    self.current_distance_x,
                    self.current_distance_y,
                    self.current_f,
                );

                self.current_startnanos = self._now;
                self.current_endnanos = self
                    ._now
                    .wrapping_add(libm::roundf(self.current_duration) as u64);
                self.is_move_first_interpolation = false;
            }
        }

        //Actual interpolation
        if self._now >= self.current_endnanos {
            //done interpolating
            self.current_from_x = self.current_to_x;
            self.current_from_y = self.current_to_y;
            self.current_cmd_x = self.current_to_x;
            self.current_cmd_y = self.current_to_y;
            self._status = MotionStatus::IDLE;
            self.is_move_first_interpolation = true;
            return self._now == self.current_endnanos;
        } else {
            let fraction_of_move =
                self._now.wrapping_sub(self.current_startnanos) as f32 / self.current_duration;
            self.current_cmd_x = self.current_from_x + (self.current_distance_x * fraction_of_move);
            self.current_cmd_y = self.current_from_y + (self.current_distance_y * fraction_of_move);
            return true;
        }
    }

    fn set_galvo_position(&mut self, x: f32, y: f32) {
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
        (self.master.value64().0 as f32 * self.to_nanos) as u64
    }
}

fn calculate_move_length_nanos(xdist: f32, ydist: f32, move_velocity: f32) -> f32 {
    let length_of_move = libm::sqrtf(xdist * xdist + ydist * ydist);
    length_of_move * 1000.0 * 1000.0 * 1000.0 / move_velocity
}
