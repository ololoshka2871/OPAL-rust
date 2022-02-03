#![no_std]

pub mod stm32l4x3;

mod hal;
pub use hal::qspi;
