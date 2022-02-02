use crate::hal::rcc as override_rcc;
use crate::stm32l4x3 as override_pac;

use crate::hal::rcc::Enable;

macro_rules! bus_enable {
    ($PER:ident => $en:ident) => {
        impl override_rcc::Enable for override_pac::$PER {
            #[inline(always)]
            fn enable(bus: &mut Self::Bus) {
                bus.enr().modify(|_, w| w.$en().set_bit());
                // Stall the pipeline to work around erratum 2.1.13 (DM00037591)
                cortex_m::asm::dsb(); // TODO: check if needed
            }
            #[inline(always)]
            fn disable(bus: &mut Self::Bus) {
                bus.enr().modify(|_, w| w.$en().clear_bit());
            }
            #[inline(always)]
            fn is_enabled() -> bool {
                Self::Bus::new().enr().read().$en().bit_is_set()
            }
            #[inline(always)]
            fn is_disabled() -> bool {
                Self::Bus::new().enr().read().$en().bit_is_clear()
            }
            #[inline(always)]
            unsafe fn enable_unchecked() {
                Self::enable(&mut Self::Bus::new());
            }
            #[inline(always)]
            unsafe fn disable_unchecked() {
                Self::disable(&mut Self::Bus::new());
            }
        }
    };
}

macro_rules! bus_smenable {
    ($PER:ident => $smen:ident) => {
        impl override_rcc::SMEnable for override_pac::$PER {
            #[inline(always)]
            fn enable_in_sleep_mode(bus: &mut Self::Bus) {
                bus.smenr().modify(|_, w| w.$smen().set_bit());
                // Stall the pipeline to work around erratum 2.1.13 (DM00037591)
                cortex_m::asm::dsb();
            }
            #[inline(always)]
            fn disable_in_sleep_mode(bus: &mut Self::Bus) {
                bus.smenr().modify(|_, w| w.$smen().clear_bit());
            }
            #[inline(always)]
            fn is_enabled_in_sleep_mode() -> bool {
                Self::Bus::new().smenr().read().$smen().bit_is_set()
            }
            #[inline(always)]
            fn is_disabled_in_sleep_mode() -> bool {
                Self::Bus::new().smenr().read().$smen().bit_is_clear()
            }
            #[inline(always)]
            unsafe fn enable_in_sleep_mode_unchecked() {
                Self::enable(&mut Self::Bus::new());
            }
            #[inline(always)]
            unsafe fn disable_in_sleep_mode_unchecked() {
                Self::disable(&mut Self::Bus::new());
            }
        }
    };
}
macro_rules! bus_reset {
    ($PER:ident => $rst:ident) => {
        impl override_rcc::Reset for override_pac::$PER {
            #[inline(always)]
            fn reset(bus: &mut Self::Bus) {
                bus.rstr().modify(|_, w| w.$rst().set_bit());
                bus.rstr().modify(|_, w| w.$rst().clear_bit());
            }
            #[inline(always)]
            unsafe fn reset_unchecked() {
                Self::reset(&mut Self::Bus::new());
            }
        }
    };
}

macro_rules! bus {
    ($($PER:ident => ($busX:ty, $($en:ident)?, $($smen:ident)?, $($rst:ident)?),)+) => {
        $(
            impl crate::hal::sealed::Sealed for override_pac::$PER {}
            impl override_rcc::RccBus for override_pac::$PER {
                type Bus = $busX;
            }
            $(bus_enable!($PER => $en);)?
            $(bus_smenable!($PER => $smen);)?
            $(bus_reset!($PER => $rst);)?
        )+
    };
}

bus! {
    QUADSPI => (override_rcc::AHB3, qspien, qspismen, qspirst), // 8
}
