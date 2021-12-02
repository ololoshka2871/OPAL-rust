// see: src/config/FreeRTOSConfig.h: configMAX_SYSCALL_INTERRUPT_PRIORITY
pub const IRQ_HIGEST_PRIO: u8 = 80;

/// USB interrupt ptiority
pub const USB_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO + 1;

/// master counter interrupt prio
pub const MASTER_COUNTER_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO + 20;

//-----------------------------------------------------------------------------

/// pseudo-idle task prio
pub const PSEOUDO_IDLE_TASK_PRIO: u8 = 1;

/// usbd task prio
pub const USBD_TASK_PRIO: u8 = PSEOUDO_IDLE_TASK_PRIO + 2;

/// protobuf task prio
pub const PROTOBUF_TASK_PRIO: u8 = PSEOUDO_IDLE_TASK_PRIO + 1;

/// monitor task prio
pub const MONITOR_TASK_PRIO: u8 = PSEOUDO_IDLE_TASK_PRIO + 1;

/// sensor processor task prio
pub const SENS_PROC_TASK_PRIO: u8 = PSEOUDO_IDLE_TASK_PRIO + 1;

//-----------------------------------------------------------------------------

pub const INITIAL_FREQMETER_TARGET: u32 = 100;
