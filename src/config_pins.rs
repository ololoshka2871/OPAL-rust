use stm32l4xx_hal::gpio::PinState;

//-----------------------------------------------------------------------------

// generator enable/disable lvls
pub const GENERATOR_ENABLE_LVL: PinState = PinState::High;
pub const GENERATOR_DISABLE_LVL: PinState = PinState::Low;

// Led
pub const LED_DISABLE: PinState = PinState::High;
pub const LED_ENABLE: PinState = PinState::Low;

//-----------------------------------------------------------------------------
