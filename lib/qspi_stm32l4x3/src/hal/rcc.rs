use stm32l4xx_hal::stm32::{rcc, RCC};
//use stm32l4xx_hal::rcc;

use crate::hal::Sealed;

// нельзя использовать stm32l4xx_hal::rcc::bus_struct - приватная
macro_rules! bus_struct {
    ($($busX:ident => ($EN:ident, $en:ident, $SMEN:ident, $smen:ident, $RST:ident, $rst:ident, $doc:literal),)+) => {
        $(
            #[doc = $doc]
            pub struct $busX {
                _0: (),
            }

            impl $busX {
                pub(crate) fn new() -> Self {
                    Self { _0: () }
                }

                #[allow(unused)]
                pub(crate) fn enr(&self) -> &rcc::$EN {
                    // NOTE(unsafe) this proxy grants exclusive access to this register
                    unsafe { &(*RCC::ptr()).$en }
                }

                #[allow(unused)]
                pub(crate) fn smenr(&self) -> &rcc::$SMEN {
                    // NOTE(unsafe) this proxy grants exclusive access to this register
                    unsafe { &(*RCC::ptr()).$smen }
                }

                #[allow(unused)]
                pub(crate) fn rstr(&self) -> &rcc::$RST {
                    // NOTE(unsafe) this proxy grants exclusive access to this register
                    unsafe { &(*RCC::ptr()).$rst }
                }
            }
        )+
    };
}

/// Bus associated to peripheral
 
// Дальше все связано с Sealed, поэтому тащим все с собой.
pub trait RccBus: Sealed {
    /// Bus type;
    type Bus;
}

/// Enable/disable peripheral
pub trait Enable: RccBus {
    /// Enables peripheral
    fn enable(bus: &mut Self::Bus);

    /// Disables peripheral
    fn disable(bus: &mut Self::Bus);

    /// Check if peripheral enabled
    fn is_enabled() -> bool;

    /// Check if peripheral disabled
    fn is_disabled() -> bool;

    /// # Safety
    ///
    /// Enables peripheral. Takes access to RCC internally
    unsafe fn enable_unchecked();

    /// # Safety
    ///
    /// Disables peripheral. Takes access to RCC internally
    unsafe fn disable_unchecked();
}

/// Reset peripheral
pub trait Reset: RccBus {
    /// Resets peripheral
    fn reset(bus: &mut Self::Bus);

    /// # Safety
    ///
    /// Resets peripheral. Takes access to RCC internally
    unsafe fn reset_unchecked();
}

/// Enable/disable peripheral in sleep mode
pub trait SMEnable: RccBus {
    /// Enables peripheral
    fn enable_in_sleep_mode(bus: &mut Self::Bus);

    /// Disables peripheral
    fn disable_in_sleep_mode(bus: &mut Self::Bus);

    /// Check if peripheral enabled
    fn is_enabled_in_sleep_mode() -> bool;

    /// Check if peripheral disabled
    fn is_disabled_in_sleep_mode() -> bool;

    /// # Safety
    ///
    /// Enables peripheral. Takes access to RCC internally
    unsafe fn enable_in_sleep_mode_unchecked();

    /// # Safety
    ///
    /// Disables peripheral. Takes access to RCC internally
    unsafe fn disable_in_sleep_mode_unchecked();
}

bus_struct! {
    AHB3 => (AHB3ENR, ahb3enr, AHB3SMENR, ahb3smenr, AHB3RSTR, ahb3rstr, "Advanced High-performance Bus 3 (AHB3) registers"),
}
