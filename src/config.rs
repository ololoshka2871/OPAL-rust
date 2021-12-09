use stm32l4xx_hal::gpio::State;

//-----------------------------------------------------------------------------

pub const XTAL_FREQ: u32 = 12_000_000;

//-----------------------------------------------------------------------------

// see: src/config/FreeRTOSConfig.h: configMAX_SYSCALL_INTERRUPT_PRIORITY
// value + -> prio -
pub const IRQ_HIGEST_PRIO: u8 = 80;

/// master counter interrupt prio
pub const MASTER_COUNTER_INTERRUPT_PRIO: u8 = IRQ_HIGEST_PRIO + 20;

/// USB interrupt ptiority
pub const USB_INTERRUPT_PRIO: u8 = MASTER_COUNTER_INTERRUPT_PRIO + 1;

// dma value captured interrupt prio
pub const DMA_IRQ_PRIO: u8 = IRQ_HIGEST_PRIO + 5;

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

pub const INITIAL_FREQMETER_TARGET: u32 = 50;

//-----------------------------------------------------------------------------

// generator enable/disable lvls
pub const GENERATOR_ENABLE_LVL: State = State::High;
pub const GENERATOR_DISABLE_LVL: State = State::Low;

//-----------------------------------------------------------------------------

pub const MINIMUM_ADAPTATION_INTERVAL: u32 = 50;

//-----------------------------------------------------------------------------
