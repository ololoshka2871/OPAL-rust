//-----------------------------------------------------------------------------

pub const XTAL_FREQ: u32 = 12_000_000;

//-----------------------------------------------------------------------------

pub const FREERTOS_CONFIG_FREQ: u32 = 72_000_000;

//-----------------------------------------------------------------------------

// see: src/config/FreeRTOSConfig.h: configMAX_SYSCALL_INTERRUPT_PRIORITY
// value + -> prio -
pub const IRQ_HIGEST_PRIO: u8 = 80;

/// master counter interrupt prio
pub const MASTER_COUNTER_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO + 10;

/// USB interrupt ptiority
pub const USB_INTERRUPT_PRIO: u8 = MASTER_COUNTER_INTERRUPT_PRIO + 1;

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

//-----------------------------------------------------------------------------
