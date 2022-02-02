#![no_std]

pub(crate) mod stm32l4x3;

#[cfg(feature = "stm32l4x3")]
mod hal;
#[cfg(feature = "stm32l4x3")]
pub use hal::qspi;
