pub mod rcc;
pub mod enable;
pub mod qspi;

mod sealed {
    pub trait Sealed {}
}
pub(crate) use sealed::Sealed;