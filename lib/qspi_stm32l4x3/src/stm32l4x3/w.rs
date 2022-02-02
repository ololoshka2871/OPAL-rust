use core::marker;

use stm32l4xx_hal::stm32l4::RegisterSpec;

/// Register writer.
///
/// Used as an argument to the closures in the `write` and `modify` methods of the register.
pub struct W<REG: RegisterSpec + ?Sized> {
    ///Writable bits
    pub(crate) bits: REG::Ux,
    _reg: marker::PhantomData<REG>,
}

impl<REG: RegisterSpec> W<REG> {
    /// Writes raw bits to the register.
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: REG::Ux) -> &mut Self {
        self.bits = bits;
        self
    }
}