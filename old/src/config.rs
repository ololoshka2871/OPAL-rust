//-----------------------------------------------------------------------------

pub const XTAL_FREQ: u32 = 16_000_000;

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

/// for freeRTOS
pub const MAX_TASK_NAME_LEN: usize = 8;

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
pub const MOTIOND_TASK_PRIO: u8 = IDLE_TASK_PRIO + 1;

//-----------------------------------------------------------------------------

/// monitor stack size
pub const USBD_TASK_STACK_SIZE: usize = 1024;

/// monitor stack size
pub const MONITOR_TASK_STACK_SIZE: usize = 2048 + 2048;

/// motion stack size
pub const MOTION_TASK_STACK_SIZE: usize = 1024;

/// gcode stack size
pub const G_CODE_TASK_STACK_SIZE: usize = 1024;

//-----------------------------------------------------------------------------

// usb pull up
pub const USB_PULLUP_ACTVE_LEVEL: bool = false;

//-----------------------------------------------------------------------------

/// max laser S 100 -> 100%
pub const MOTION_MAX_S: f32 = 100f32;

/// working range X
pub const MOTION_X_RANGE: f32 = 250.0;

/// working range Y
pub const MOTION_Y_RANGE: f32 = 250.0;

/// working range Z (unused)
pub const MOTION_Z_RANGE: f32 = 1.0;

/// invert axis
pub const AXIS_INVERSE_X: bool = false;
pub const AXIS_INVERSE_Y: bool = false;

//-----------------------------------------------------------------------------

/// galvo power enable active lvl
pub const GALVO_EN_ACTIVE_LVL: bool = true;

/// laser power enable active lvl
pub const LASER_EN_ACTIVE_LVL: bool = true;

//-----------------------------------------------------------------------------

/// main laser sync frequency - from laser head docs
pub const LASER_SYNC_CLOCK_KHZ: u32 = 45;

/// red mark laser pwm frequency
pub const LASER_RED_FREQ_KHZ: u32 = 1;
