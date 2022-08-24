use core::str::FromStr;

use alloc::{fmt::format, string::String};

pub const MAX_LEN: usize = 150;

#[derive(Clone, Copy)]
pub enum Code {
    G(u32),
    M(u32),
    Empty,
}

#[derive(Clone, Copy)]
pub struct GCode {
    code: Code,

    x: Option<f64>,
    y: Option<f64>,

    s: Option<f64>, // Laser Power
    f: Option<f64>, // FeedRate
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
                new_code.code = Code::M(
                    Self::search_string('M', text)
                        .or_else(|_| Err(ParceError::Error("Failed to parse M command".into())))?,
                );

                new_code.s = Self::get_val('S', text)
                    .or_else(|_| Err(ParceError::Error("Invalid S value".into())))?;
            } else if Self::has_command('G', text) {
                new_code.code = Code::G(
                    Self::search_string('G', text)
                        .or_else(|_| Err(ParceError::Error("Failed to parse M command".into())))?,
                );

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

    fn search_string<T: FromStr>(key: char, text: &str) -> Result<T, T::Err> {
        text.chars()
            .skip_while(|c| *c != key)
            .skip(1)
            .take_while(|c| ['.', '+', '-'].contains(c) || c.is_numeric())
            .collect::<String>()
            .parse()
    }

    fn get_val<T: FromStr>(key: char, text: &str) -> Result<Option<T>, T::Err> {
        if Self::has_command(key, text) {
            Ok(Some(Self::search_string(key, text)?))
        } else {
            Ok(None)
        }
    }

    fn fill_letters(&mut self, text: &str) -> Result<(), ParceError> {
        for (field, letter) in
            [&mut self.x, &mut self.y, &mut self.f, &mut self.s].zip(['X', 'Y', 'F', 'S'])
        {
            *field = Self::get_val(letter, text).or_else(|_| {
                Err(ParceError::Error(format(format_args!(
                    "Failed to parse {} value",
                    letter
                ))))
            })?;
        }
        Ok(())
    }

    #[inline]
    pub fn code(&self) -> Code {
        self.code
    }

    #[inline]
    pub fn get_x(&self) -> Option<f64> {
        self.x
    }

    #[inline]
    pub fn get_y(&self) -> Option<f64> {
        self.y
    }

    #[inline]
    pub fn get_s(&self) -> Option<f64> {
        self.s
    }

    #[inline]
    pub fn get_f(&self) -> Option<f64> {
        self.f
    }
}

impl Default for GCode {
    fn default() -> Self {
        Self {
            code: Code::Empty,
            x: None,
            y: None,
            s: None,
            f: None,
        }
    }
}
