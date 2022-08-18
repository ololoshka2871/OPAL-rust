//-----------------------------------------------------------------------------

pub const XTAL_FREQ: u32 = 12_000_000;

//-----------------------------------------------------------------------------

pub const FREERTOS_CONFIG_FREQ: u32 = 72_000_000;

pub const GALVO_CLOCK_RATE: u32 = 2_000_000 * 2; // clock needs 2 ticks

//-----------------------------------------------------------------------------

// see: src/config/FreeRTOSConfig.h: configMAX_SYSCALL_INTERRUPT_PRIORITY
// value + -> prio -
pub const IRQ_HIGEST_PRIO: u8 = 80;

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

// main motion task, prio same as idle
pub const MOTIOND_TASK_PRIO: u8 = IDLE_TASK_PRIO;

/// usbd task prio
pub const USBD_TASK_PRIO: u8 = IDLE_TASK_PRIO + 3;

/// monitor task prio
pub const MONITOR_TASK_PRIO: u8 = IDLE_TASK_PRIO + 1;

/// G-Code task prio
pub const GCODE_TASK_PRIO: u8 = IDLE_TASK_PRIO + 2;

//-----------------------------------------------------------------------------
