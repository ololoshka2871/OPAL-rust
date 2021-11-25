// see: src/config/FreeRTOSConfig.h: configMAX_SYSCALL_INTERRUPT_PRIORITY
pub const IRQ_HIGEST_PRIO: u8 = 80;

/// USB interrupt ptiority
pub const USB_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO + 1;

/// master counter interrupt prio
pub const MASTER_COUNTER_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO;
