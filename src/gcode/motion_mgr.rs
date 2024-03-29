use core::fmt::{Display, Write};
use core::str::FromStr;

use crate::config;
use crate::config::HlString as String;

pub type LongString = heapless::String<512>;

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
    current_f: f32, // mm/min
    current_s: u8,
    current_a: f32,
    current_b: u32,
    current_duration: f32,
    current_absolute: bool,
    current_laserenabled: bool,
    current_red_laserenabled: bool,
    laser_changed: bool,

    avlb: usize,

    laser: LASER,
    galvo: GALVO,
}

impl<LASER, GALVO> MotionMGR<LASER, GALVO>
where
    GALVO: crate::control::xy2_100::XY2_100Interface,
    LASER: crate::control::laser::LaserInterface,
{
    pub fn new(galvo: GALVO, laser: LASER, buf_sz: usize) -> Self {
        Self {
            _status: MotionStatus::IDLE,
            is_move_first_interpolation: true,
            current_startnanos: 0,
            current_endnanos: 0,
            _now: 0,
            current_code: 0,
            current_from_x: 0.0,
            current_from_y: 0.0,
            current_distance_x: 0.0,
            current_distance_y: 0.0,
            current_to_x: 0.0,
            current_to_y: 0.0,
            current_cmd_x: 0.0,
            current_cmd_y: 0.0,
            current_f: 100.0,
            current_s: 0,
            current_a: 100.0,
            current_b: crate::config::LASER_SYNC_CLOCK_KHZ * 1000,
            current_duration: 0.0,
            current_absolute: true,
            current_laserenabled: false,
            current_red_laserenabled: false,
            laser_changed: false,

            avlb: buf_sz,

            laser,
            galvo,
        }
    }

    pub fn begin(&mut self) {
        self.set_galvo_position(0.0, 0.0);
    }

    pub fn is_busy(&self) -> bool {
        self._status != MotionStatus::IDLE
    }

    pub fn process(&mut self, gcode: &mut GCode, avlb: usize) -> Result<Option<String>, String> {
        self.avlb = avlb;
        if self._status == MotionStatus::IDLE {
            use super::gcode::Code;
            match gcode.code() {
                Code::G(_) => {
                    self.process_gcodes(gcode)?;
                    Ok(None)
                }
                Code::M(code) => self.process_mcodes(code, gcode),
                Code::Empty => {
                    self.process_other(gcode)?;
                    Ok(None)
                }
            }
        } else {
            Err("Motion busy!".into())
        }
    }

    pub fn tic(&mut self, now_nanos: u64) -> MotionStatus {
        self._now = now_nanos;
        if self._status == MotionStatus::INTERPOLATING {
            if self.interpolate_move() {
                self.set_galvo_position(self.current_cmd_x, self.current_cmd_y);
            }
        }

        if self.laser_changed {
            if self.current_laserenabled {
                self.laser.set_pump_power(self.current_s);
                self.laser.set_frequency(self.current_b as u32);
                self.laser.set_power_pwm(self.current_a);
                self.laser.enable();
            } else {
                self.laser.disable()
            }

            if self.current_red_laserenabled {
                self.laser.set_red_laser_power(self.current_a)
            } else {
                self.laser.set_red_laser_power(0.0);
            }

            self.laser_changed = false;
        }

        self._status
    }

    fn process_gcodes(&mut self, gcode: &mut GCode) -> Result<(), String> {
        use super::gcode::Code;

        if gcode.code() == Code::Empty {
            gcode.set_code(Code::G(if self.current_code == 0 { 0 } else { 1 }));
        }

        match gcode.code() {
            Code::G(0) => {
                self.current_code = 0;
                self.set_xyab(&gcode)?;
            }
            Code::G(1) => {
                self.current_code = 1;

                if let Some(new_s) = gcode.get_s() {
                    if let Err(_) = Self::set_value(
                        &mut self.current_s,
                        new_s as u8,
                        'S',
                        config::MOTION_MAX_S as u8,
                        0,
                    ) {
                        if new_s > config::MOTION_MAX_S {
                            self.current_s = config::MOTION_MAX_S as u8;
                        } else {
                            self.current_s = 0;
                        }
                    }
                }
                if let Some(new_f) = gcode.get_f() {
                    Self::set_value(&mut self.current_f, new_f, 'F', i32::MAX as f32, 0.01f32)?;
                }

                self.set_xyab(&gcode)?;
            }

            Code::G(28) => {
                self.current_code = 28;
                self.current_to_x = 0.0;
                self.current_to_y = 0.0;
            }
            Code::G(90) => {
                self.current_absolute = true;
                return Ok(());
            }
            Code::G(91) => {
                self.current_absolute = false;

                // ignore F
                if gcode.get_x().is_some() || gcode.get_y().is_some() {
                    self.current_code = 0;
                    self.set_xyab(&gcode)?;
                }
            }
            Code::G(94) => {
                // подача мм/мин.
                return Ok(());
            }

            // g54 - система координат
            // g17 - плосткость XY
            // G20/G21 - дюймы/милиметры
            // g43[.*] - смещеине инструмента
            // g49 - отмена коррекции длины инструмента
            _ => return Ok(()),
        }

        self._status = MotionStatus::INTERPOLATING;

        Ok(())
    }

    fn set_xyab(&mut self, gcode: &GCode) -> Result<(), String> {
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
            if let Err(_) = Self::set_value(&mut self.current_a, new_a, 'A', 100.0, 0.0) {
                if new_a > 100.0 {
                    self.current_a = 100.0; // перебор
                } else {
                    self.current_a = 0.0; // < 0
                }
            }
        }

        if let Some(new_b) = gcode.get_b() {
            if let Err(_) = Self::set_value(&mut self.current_b, new_b as u32, 'B', 80000, 20000) {
                if new_b > 80000.0 {
                    self.current_b = 80000; // перебор
                } else {
                    self.current_b = 20000; // недобор
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
            let mut s = String::new();
            write!(&mut s, "{} above limit", name).unwrap();
            Err(s)
        } else if src < nlimit {
            let mut s = String::new();
            write!(&mut s, "{} below limit", name).unwrap();
            Err(s)
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

    fn process_mcodes(&mut self, code: u32, gcode: &GCode) -> Result<Option<String>, String> {
        match code {
            2 => {
                if self.is_busy() {
                    return Err(unsafe {
                        String::from_str("[MSG:Pgm End]\r\n").unwrap_unchecked()
                    });
                } else {
                    // костыль, почему-то где-то теряется 3 ok'а
                    return Ok(Some(unsafe {
                        String::from_str("ok\r\nok\r\nok\r\nok\r\n").unwrap_unchecked()
                    }));
                }
            }
            3 | 4 => {
                if let Some(new_s) = gcode.get_s() {
                    if let Err(_) = Self::set_value(
                        &mut self.current_s,
                        new_s as u8,
                        'S',
                        config::MOTION_MAX_S as u8,
                        0,
                    ) {
                        if new_s > config::MOTION_MAX_S {
                            self.current_s = config::MOTION_MAX_S as u8;
                        } else {
                            self.current_s = 0;
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
        Ok(None)
    }

    fn process_other(&mut self, gcode: &mut GCode) -> Result<(), String> {
        match self.current_code {
            0 | 1 => self.process_gcodes(gcode),
            c => {
                let mut s = String::new();
                write!(&mut s, "Cannot continue move for G{}", c).unwrap();
                Err(s)
            }
        }
    }

    pub fn process_status_req(
        &mut self,
        req: &super::Request,
    ) -> Result<Option<LongString>, String> {
        use super::Request;
        use crate::support::format_float_simple;
        let ok = Some(LongString::from_str("ok\r\n").unwrap());

        match req {
            Request::Dollar('G') => {
                let mut s = LongString::new();
                write!(
                    &mut s,
                    "[GC:G{g1} G54 G17 G21 G9{g9} G94 M5 M9 T0 F{f} S{s}]\r\nok\r\n",
                    g1 = self.current_code,
                    g9 = (!self.current_absolute as u32),
                    s = self.current_s,
                    f = format_float_simple(self.current_f, 3),
                )
                .unwrap();
                Ok(Some(s))
            }
            Request::Dollar('#') => {
                let mut s = LongString::new();
                /*"[G54:0.000,0.000,0.000]\r
                [G55:0.000,0.000,0.000]\r
                [G56:0.000,0.000,0.000]\r
                [G57:0.000,0.000,0.000]\r
                [G58:0.000,0.000,0.000]\r
                [G59:0.000,0.000,0.000]\r
                [G28:0.000,0.000,0.000]\r
                [G30:0.000,0.000,0.000]\r
                [G92:0.000,0.000,0.000]\r
                [TLO:0.000]\r
                [PRB:0.000,0.000,0.000:0]\r
                ok\r\n"*/
                write!(
                    &mut s,
                    "[G54:0.000,0.000,0.000]\r
[G55:0.000,0.000,0.000]\r
[G56:0.000,0.000,0.000]\r
[TLO:0.000]\r
[PRB:0.000,0.000,0.000:0]\r
ok\r\n"
                )
                .unwrap();
                Ok(Some(s))
            }
            Request::Dollar('X') => {
                // unlock
                Ok(ok)
            }
            Request::Status => {
                let mut s = LongString::new();
                write!(
                    &mut s,
                    "<{state}|MPos:{x:.3},{y:.3},0.000|Bf:{bf},150|FS:{f},{s}>\r\n",
                    state = if self.is_busy() { "Run" } else { "Idle" },
                    x = format_float_simple(self.current_cmd_x, 3),
                    y = format_float_simple(self.current_cmd_y, 3),
                    bf = self.avlb,
                    s = self.current_s,
                    f = format_float_simple(self.current_f, 3),
                )
                .unwrap();
                Ok(Some(s))
            }
            Request::Dollar(dl) => {
                let mut s = String::new();
                write!(&mut s, "Unsupported command ${}\r\n", dl).unwrap();
                Err(s)
            }
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

    pub fn debug_set_red_laser(&mut self, v: bool) {
        self.laser
            .set_red_laser_power(if v { config::MOTION_MAX_S } else { 0.0 });
    }

    pub fn debug_set_laser_enable(&mut self, v: bool) {
        self.laser.debug_set_ee(v);
    }
}

fn calculate_move_length_nanos(xdist: f32, ydist: f32, move_velocity: f32) -> f32 /* [ns] */ {
    let length_of_move = libm::sqrtf(xdist * xdist + ydist * ydist);
    1_000_000_000.0 /*[ns/s]*/ * length_of_move /*[mm]*/ / (move_velocity /*[mm/min]*/ / 60.0 /*[s/min]*/)
}
