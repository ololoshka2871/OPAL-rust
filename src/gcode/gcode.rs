use core::fmt::Write;
use core::str::FromStr;

use crate::config::HlString;

pub const MAX_LEN: usize = 150;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Code {
    G(u32),
    M(u32),
    Empty,
}

#[derive(Clone, Copy, Debug)]
pub enum Request {
    Dollar(char),
    Status,
}

#[derive(Clone, Copy, Debug)]
pub struct GCode {
    code: Code,

    x: Option<f32>,
    y: Option<f32>,
    a: Option<f32>, // Laser pump Power

    s: Option<f32>, // Laser pwm Power
    f: Option<f32>, // FeedRate
}

pub enum ParceResult {
    GCode(GCode),
    Request(Request),
    Partial(GCode, usize),
}

pub enum ParceError {
    Empty,
    Error(HlString),
}

impl GCode {
    pub fn from_string<const N: usize>(text: &str) -> Result<ParceResult, ParceError> {
        if text.is_empty() {
            return Err(ParceError::Empty);
        }
        let upper_text = text
            .chars()
            .map(|mut c| {
                c.make_ascii_uppercase();
                c
            })
            .collect::<heapless::String<N>>();
        Self::from_string_private::<N>(upper_text.as_str())
    }

    fn from_string_private<const N: usize>(text: &str) -> Result<ParceResult, ParceError> {
        let first_char = text.chars().nth(0).unwrap_or_default();
        if ['/', '(', ':'].contains(&first_char) {
            Err(ParceError::Empty)
        } else if first_char == '%' {
            Err(ParceError::Error(unsafe {
                HlString::from_str("error: 1").unwrap_unchecked()
            }))
        } else if ['?', '$'].contains(&first_char) {
            if Self::has_command('$', text) {
                if Self::has_command('J', text) {
                    // Jog
                    let mut new_code = Self::default();
                    if Self::has_command('G', text) {
                        new_code.code = Code::G(Self::search_value('G', text).or_else(|_| {
                            Err(ParceError::Error("Failed to parse M command".into()))
                        })?);
                        new_code.fill_letters(text)?;

                        Ok(ParceResult::GCode(new_code))
                    } else {
                        Err(ParceError::Error(HlString::from_str("jog error").unwrap()))
                    }
                } else {
                    Ok(ParceResult::Request(Request::Dollar(
                        match { text.chars().skip_while(|c| *c != '$').skip(1).next() } {
                            Some(c) => c,
                            None => Err(ParceError::Error("Failed to parse $ command".into()))?,
                        },
                    )))
                }
            } else {
                Ok(ParceResult::Request(Request::Status))
            }
        } else {
            let mut new_code = Self::default();

            if Self::has_command('M', text) {
                new_code.code = Code::M(
                    Self::search_value('M', text)
                        .or_else(|_| Err(ParceError::Error("Failed to parse M command".into())))?,
                );

                new_code.s = Self::get_val('S', text)
                    .or_else(|_| Err(ParceError::Error("Invalid S value".into())))?;
            } else if Self::has_command('G', text) {
                let command_number = Self::search_value::<f32>('G', text)
                    .or_else(|_| Err(ParceError::Error("Failed to parse Gcode number".into())))?;
                if command_number > 43.0 && command_number < 43.9 {
                    new_code.code = Code::G(43); // дробная часть не интересна
                } else {
                    new_code.code = Code::G(command_number as u32);
                }

                if new_code.code == Code::G(90) || new_code.code == Code::G(91) {
                    return Ok(ParceResult::Partial(new_code, 3));
                }

                new_code.fill_letters(text)?;
            } else {
                new_code.fill_letters(text)?;
            }
            Ok(ParceResult::GCode(new_code))
        }
    }

    #[inline]
    fn has_command(key: char, text: &str) -> bool {
        text.contains(key)
    }

    fn search_value<T: FromStr>(key: char, text: &str) -> Result<T, T::Err> {
        text.chars()
            .skip_while(|c| *c != key)
            .skip(1)
            .take_while(|c| ['.', '+', '-'].contains(c) || c.is_numeric())
            .collect::<HlString>()
            .parse()
    }

    fn get_val<T: FromStr>(key: char, text: &str) -> Result<Option<T>, T::Err> {
        if Self::has_command(key, text) {
            Ok(Some(Self::search_value(key, text)?))
        } else {
            Ok(None)
        }
    }

    fn fill_letters(&mut self, text: &str) -> Result<(), ParceError> {
        for (field, letter) in [
            &mut self.x,
            &mut self.y,
            &mut self.a,
            &mut self.f,
            &mut self.s,
        ]
        .zip(['X', 'Y', 'A', 'F', 'S'])
        {
            *field = Self::get_val(letter, text).or_else(|_| {
                let mut str = HlString::new();
                let _ = write!(&mut str, r#"Failed to parse {} value "{}""#, letter, text);
                Err(ParceError::Error(str))
            })?;
        }
        Ok(())
    }

    #[inline]
    pub fn code(&self) -> Code {
        self.code
    }

    #[inline]
    pub fn set_code(&mut self, code: Code) {
        self.code = code;
    }

    #[inline]
    pub fn get_x(&self) -> Option<f32> {
        self.x
    }

    #[inline]
    pub fn get_y(&self) -> Option<f32> {
        self.y
    }

    #[inline]
    pub fn get_s(&self) -> Option<f32> {
        self.s
    }

    #[inline]
    pub fn get_a(&self) -> Option<f32> {
        self.a
    }

    #[inline]
    pub fn get_f(&self) -> Option<f32> {
        self.f
    }
}

impl Default for GCode {
    fn default() -> Self {
        Self {
            code: Code::Empty,
            x: None,
            y: None,
            a: None,

            s: None,
            f: None,
        }
    }
}
