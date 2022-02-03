pub mod rcc;
pub mod enable;
pub mod qspi;

// нельзя использовать stm32l4xx_hal::sealed - приватная
mod sealed {
    pub trait Sealed {}
}
pub(crate) use sealed::Sealed;