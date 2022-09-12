use alloc::format;
use alloc::string::String;
use core::str::FromStr;

pub const MAX_LEN: usize = 150;

#[derive(Clone, Copy)]
pub enum Code {
    G(u32),
    M(u32),
    Empty,
}

#[derive(Clone, Copy)]
pub enum Request {
    Dollar(char),
    Status,
}

#[derive(Clone, Copy)]
pub struct GCode {
    code: Code,

    x: Option<f32>,
    y: Option<f32>,

    s: Option<f32>, // Laser pwm Power
    p: Option<f32>, // Laser pump Power
    f: Option<f32>, // FeedRate
}

pub enum ParceResult {
    GCode(GCode),
    Request(Request),
}

pub enum ParceError {
    Empty,
    Error(String),
}

impl GCode {
    pub fn from_string(text: &str) -> Result<ParceResult, ParceError> {
        let upper_text = text.to_uppercase();
        let text = upper_text.as_str();
        let first_char = text.chars().nth(0).unwrap_or_default();
        if ['/', '(', ':'].contains(&first_char) {
            Err(ParceError::Empty)
        } else if ['?', '$'].contains(&first_char) {
            Ok(if Self::has_command('$', text) {
                ParceResult::Request(Request::Dollar(
                    match { text.chars().skip_while(|c| *c != '$').skip(1).next() } {
                        Some(c) => c,
                        None => Err(ParceError::Error("Failed to parse $ command".into()))?,
                    },
                ))
            } else {
                ParceResult::Request(Request::Status)
            })
        } else {
            let mut new_code = Self::default();

            if Self::has_command('M', text) {
                new_code.code = Code::M(
                    Self::search_value('M', text)
                        .or_else(|_| Err(ParceError::Error("Failed to parse M command".into())))?,
                );

                new_code.s = Self::get_val('S', text)
                    .or_else(|_| Err(ParceError::Error("Invalid S value".into())))?;
                new_code.p = Self::get_val('P', text)
                    .or_else(|_| Err(ParceError::Error("Invalid P value".into())))?;
            } else if Self::has_command('G', text) {
                new_code.code = Code::G(
                    Self::search_value('G', text)
                        .or_else(|_| Err(ParceError::Error("Failed to parse M command".into())))?,
                );

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
            .collect::<String>()
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
        for (field, letter) in
            [&mut self.x, &mut self.y, &mut self.f, &mut self.s].zip(['X', 'Y', 'F', 'S'])
        {
            *field = Self::get_val(letter, text).or_else(|_| {
                Err(ParceError::Error(format!(
                    "Failed to parse {} value \"{}\"",
                    letter, text
                )))
            })?;
        }
        Ok(())
    }

    #[inline]
    pub fn code(&self) -> Code {
        self.code
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
    pub fn get_p(&self) -> Option<f32> {
        self.p
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
            s: None,
            p: None,
            f: None,
        }
    }
}
