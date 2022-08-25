//-----------------------------------------------------------------------------

pub const XTAL_FREQ: u32 = 12_000_000;

//-----------------------------------------------------------------------------

pub const FREERTOS_CONFIG_FREQ: u32 = 72_000_000;

pub const GALVO_CLOCK_RATE: u32 = 2_000_000 * 2; // clock needs 2 ticks

//-----------------------------------------------------------------------------

// see: src/config/FreeRTOSConfig.h: configMAX_SYSCALL_INTERRUPT_PRIORITY
// value + -> prio -
pub const IRQ_HIGEST_PRIO: u8 = 80;

/// master counter interrupt prio
pub const MASTER_COUNTER_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO + 10;

/// galvo interface tick prio
pub const GALVO_INTERFACE_TICK_PRIO: u8 = IRQ_HIGEST_PRIO + 2;

/// USB interrupt ptiority
pub const USB_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO + 6;

// dma value captured interrupt prio
pub const DMA_IRQ_PRIO: u8 = IRQ_HIGEST_PRIO + 5;

//-----------------------------------------------------------------------------

// Приоритеты, обольше -> лучше

/// pseudo-idle task prio
pub const IDLE_TASK_PRIO: u8 = 0;

/// usbd task prio
pub const USBD_TASK_PRIO: u8 = IDLE_TASK_PRIO + 3;

/// monitor task prio
pub const MONITOR_TASK_PRIO: u8 = IDLE_TASK_PRIO + 1;

/// G-Code task prio
pub const GCODE_TASK_PRIO: u8 = IDLE_TASK_PRIO + 2;

// main motion task, prio same as idle
pub const MOTIOND_TASK_PRIO: u8 = IDLE_TASK_PRIO;

//-----------------------------------------------------------------------------

// max laser S 100 -> 100%
pub const MOTION_MAX_S: f64 = 100f64;

// working range X
pub const MOTION_X_RANGE: f64 = 250.0;

// working range Y
pub const MOTION_Y_RANGE: f64 = 250.0;

// working range Z (unused)
pub const MOTION_Z_RANGE: f64 = 1.0;

// invert axis
pub const AXIS_INVERSE_X: bool = false;
pub const AXIS_INVERSE_Y: bool = false;

//-----------------------------------------------------------------------------

// galvo power enable active lvl
pub const GALVO_EN_ACTIVE_LVL: bool = true;

// laser power enable active lvl
pub const LASER_EN_ACTIVE_LVL: bool = true;
