pub(crate) mod field_reader;
pub(crate) mod generics;

use core::{marker::PhantomData, ops::Deref};

#[doc = "QuadSPI interface"]
pub struct QUADSPI {
    _marker: PhantomData<*const ()>,
}
unsafe impl Send for QUADSPI {}
impl QUADSPI {
    #[doc = r"Pointer to the register block"]
    pub const PTR: *const quadspi::RegisterBlock = 0xa000_1000 as *const _;
    #[doc = r"Return the pointer to the register block"]
    #[inline(always)]
    pub const fn ptr() -> *const quadspi::RegisterBlock {
        Self::PTR
    }

    pub unsafe fn new() -> Self {
        Self{
            _marker: PhantomData
        }
    }
}
impl Deref for QUADSPI {
    type Target = quadspi::RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::PTR }
    }
}
impl core::fmt::Debug for QUADSPI {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("QUADSPI").finish()
    }
}
#[doc = "QuadSPI interface"]
pub mod quadspi;
