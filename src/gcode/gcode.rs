use core::str::FromStr;

use alloc::{fmt::format, string::String};

pub const MAX_LEN: usize = 150;

#[derive(Clone, Copy)]
pub struct GCode {
    codeprefix: char,
    code: u32, // Go = 0 , G28 = 28 etc. Only G0 / G1 Supported so far...

    x: f64,
    y: f64,
    z: f64,
    /*
    e: f64,

    a: f64,
    b: f64,
    c: f64,

    i: f64,
    j: f64,

    p: f64,
    */
    s: f64, // Laser Power
    f: f64, // FeedRate
            /*
            r: f64, // Misc
            t: f64, // Misc
            */
            //move_length_nanos: f64,
            //fwd_cmd: [u8; MAX_LEN],
}

pub enum ParceError {
    Empty,
    Error(String),
}

impl GCode {
    pub fn from_string(text: &str) -> Result<GCode, ParceError> {
        let upper_text = text.to_uppercase();
        let text = upper_text.as_str();
        if ['/', '(', ':'].contains(&text.chars().nth(0).unwrap()) {
            Err(ParceError::Empty)
        } else {
            let mut new_code = Self::default();

            if Self::has_command('M', text) {
                new_code.codeprefix = 'M';
                new_code.code = Self::search_string('M', text)
                    .or_else(|_| Err(ParceError::Error("Failed to parse M command".into())))?;

                /*
                let startpos = Self::has_command_at('M', text);
                let cpy_src = &text.as_bytes()[startpos..];
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        cpy_src.as_ptr(),
                        new_code.fwd_cmd.as_mut_ptr(),
                        cpy_src.len(),
                    );
                }
                */

                new_code.s = Self::get_val('S', text, 0_f64)
                    .or_else(|_| Err(ParceError::Error("Invalid S value".into())))?;
            } else if Self::has_command('G', text) {
                new_code.codeprefix = 'G';

                new_code.code = Self::search_string('G', text)
                    .or_else(|_| Err(ParceError::Error("Failed to parse G command".into())))?;

                new_code.fill_letters(text)?;
            } else {
                new_code.fill_letters(text)?;
            }
            Ok(new_code)
        }
    }

    #[inline]
    fn has_command(key: char, text: &str) -> bool {
        text.contains(key)
    }

    /*
    #[inline]
    fn has_command_at(key: char, text: &str) -> usize {
        text.find(key).unwrap()
    }
    */

    fn search_string<T: FromStr>(key: char, text: &str) -> Result<T, T::Err> {
        text.chars()
            .skip_while(|c| *c != key)
            .skip(1)
            .take_while(|c| ['.', '+', '-'].contains(c) || c.is_numeric())
            .collect::<String>()
            .parse()
    }

    fn get_val<T: FromStr>(key: char, text: &str, default: T) -> Result<T, T::Err> {
        if Self::has_command(key, text) {
            Self::search_string(key, text)
        } else {
            Ok(default)
        }
    }

    fn fill_letters(&mut self, text: &str) -> Result<(), ParceError> {
        for (field, letter) in [
            &mut self.x,
            &mut self.y,
            &mut self.z,
            /*
            &mut self.e,
            &mut self.a,
            &mut self.b,
            &mut self.c,
            */
            &mut self.f,
            /*
            &mut self.i,
            &mut self.j,
            &mut self.p,
            &mut self.r,
            */
            &mut self.s,
            /*
            &mut self.t,
            */
        ]
        .zip([
            'X', 'Y', 'Z', /*'E', 'A', 'B', 'C',*/ 'F',
            /*'I', 'J', 'P', 'R',*/ 'S', /*'T',*/
        ]) {
            *field = Self::get_val(letter, text, 0f64).or_else(|_| {
                Err(ParceError::Error(format(format_args!(
                    "Failed to parse {} value",
                    letter
                ))))
            })?;
        }
        Ok(())
    }

    #[inline]
    pub fn is_g_code(&self) -> bool {
        self.codeprefix == 'G'
    }

    #[inline]
    pub fn is_m_code(&self) -> bool {
        self.codeprefix == 'M'
    }

    #[inline]
    pub fn code(&self) -> u32 {
        self.code
    }

    #[inline]
    pub fn get_x(&self) -> f64 {
        self.x
    }

    #[inline]
    pub fn get_y(&self) -> f64 {
        self.y
    }

    #[inline]
    pub fn get_z(&self) -> f64 {
        self.z
    }

    #[inline]
    pub fn get_s(&self) -> f64 {
        self.s
    }

    #[inline]
    pub fn get_f(&self) -> f64 {
        self.f
    }
}

/*
impl defmt::Format for GCode {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            //"{}{}\n\rX{} Y{} Z{}\n\rA{} B{} C{}\n\rI{} J{}\n\rP{}\n\rS{} F{}\n\rR{} T{}",
            "{}{}\n\rX{} Y{} Z{}\n\rS{} F{}",
            self.codeprefix,
            self.code,
            self.x,
            self.y,
            self.z,
            /*
            self.a,
            self.b,
            self.c,
            self.i,
            self.j,
            self.p,
            */
            self.s,
            self.f,
            /*
            self.r,
            self.t
            */
        )
    }
}
*/

impl Default for GCode {
    fn default() -> Self {
        Self {
            codeprefix: 'E',
            code: 0,
            x: 0f64,
            y: 0f64,
            z: 0f64,
            /*
            e: 0f64,
            a: 0f64,
            b: 0f64,
            c: 0f64,
            i: 0f64,
            j: 0f64,
            p: 0f64,
            */
            s: 0f64,
            f: 0f64,
            /*
            r: 0f64,
            t: 0f64,

            move_length_nanos: 0f64,
            fwd_cmd: [0u8; MAX_LEN],
            */
        }
    }
}
