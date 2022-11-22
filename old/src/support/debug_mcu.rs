#[doc = r"Register block"]
#[repr(C)]
pub struct RegisterBlock {
    //#[doc = "0x00 - MCU device ID code"]
    //pub idcode: IDCODE,
    pub idcode: vcell::VolatileCell<u32>,
    //#[doc = "0x04 - Debug MCU configuration register"]
    //pub cr: CR,
    pub cr: vcell::VolatileCell<u32>,
    //#[doc = "0x08 - Debug MCU APB1 freeze register 1"]
    //pub apb1fzr1: APB1FZR1,
    pub apb1fzr1: vcell::VolatileCell<u32>,
    //#[doc = "0x0c - ebug MCU APB1 freeze register 2"]
    //pub apb1fzr2: APB1FZR2,
    pub apb1fzr2: vcell::VolatileCell<u32>,
    //#[doc = "0x10 - Debug MCU APB2 freeze register"]
    //pub apb2fz: APB2FZ,
    pub apb2fz: vcell::VolatileCell<u32>,
}

pub const DEBUG_MCU: *mut crate::support::debug_mcu::RegisterBlock =
    0xE004_2000 as *mut crate::support::debug_mcu::RegisterBlock;
