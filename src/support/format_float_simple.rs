use alloc::{format, string::String};
use num::traits::float::FloatCore;

pub fn format_float_simple(v: f64, percision: i32) -> String {
    let a = v.floor();
    format!(
        "{}.{}",
        a as i32,
        ((v - a) * 10.0.powi(percision)).round() as i32
    )
}
