//-----------------------------------------------------------------------------

pub const XTAL_FREQ: u32 = 16_000_000;

//-----------------------------------------------------------------------------

pub const GALVO_CLOCK_RATE: u32 = 2_000_000 * 2; // clock needs 2 ticks

//-----------------------------------------------------------------------------

// usb pull up
pub const USB_PULLUP_ACTVE_LEVEL: bool = false;

//-----------------------------------------------------------------------------

/// gcode queue size
pub const GCODE_QUEUE_SIZE: usize = 8;

pub const STR_MAX_LEN: usize = 64;

//-----------------------------------------------------------------------------

pub const SYSTICK_RATE_HZ: u32 = 1_000;

//-----------------------------------------------------------------------------

/// max laser S 100 -> 100%
pub const MOTION_MAX_S: f32 = 100f32;

/// working range X
pub const MOTION_X_RANGE: f32 = 250.0;

/// working range Y
pub const MOTION_Y_RANGE: f32 = 250.0;

/// invert axis
pub const AXIS_INVERSE_X: bool = false;
pub const AXIS_INVERSE_Y: bool = false;

//-----------------------------------------------------------------------------

/// main laser sync frequency - from laser head docs
pub const LASER_SYNC_CLOCK_KHZ: u32 = 45;

/// red mark laser pwm frequency
pub const LASER_RED_FREQ_KHZ: u32 = 1;

//-----------------------------------------------------------------------------

pub type HlString = heapless::String<STR_MAX_LEN>;
