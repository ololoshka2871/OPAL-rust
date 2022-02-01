use stm32l4xx_hal::gpio::PinState;

//-----------------------------------------------------------------------------

pub const XTAL_FREQ: u32 = 12_000_000;
pub const FREERTOS_CONFIG_FREQ: u32 = 3_000_000; // Это же число должно быть в src/configTemplate/FreeRTOSConfig.h

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

/// pseudo-idle task prio
pub const PSEOUDO_IDLE_TASK_PRIO: u8 = 1;

/// usbd task prio
pub const USBD_TASK_PRIO: u8 = PSEOUDO_IDLE_TASK_PRIO + 3;

/// protobuf task prio
pub const PROTOBUF_TASK_PRIO: u8 = PSEOUDO_IDLE_TASK_PRIO + 1;

/// monitor task prio
pub const MONITOR_TASK_PRIO: u8 = PSEOUDO_IDLE_TASK_PRIO + 1;

/// sensor processor task prio
pub const SENS_PROC_TASK_PRIO: u8 = PSEOUDO_IDLE_TASK_PRIO + 3;

// recorder controller task prio
pub const RECORDER_CTRL_PRIO: u8 = PSEOUDO_IDLE_TASK_PRIO + 1;

/// flash cleaner prio
pub const FLASH_CLEANER_PRIO: u8 = PSEOUDO_IDLE_TASK_PRIO;

//-----------------------------------------------------------------------------

pub const INITIAL_FREQMETER_TARGET: u32 = 1;

//-----------------------------------------------------------------------------

// generator enable/disable lvls
pub const GENERATOR_ENABLE_LVL: PinState = PinState::High;
pub const GENERATOR_DISABLE_LVL: PinState = PinState::Low;

// Led
pub const LED_DISABLE: PinState = PinState::High;
pub const LED_ENABLE: PinState = PinState::Low;

//-----------------------------------------------------------------------------

pub const MINIMUM_ADAPTATION_INTERVAL: u32 = 50;
pub const MEASURE_TIME_TO_GUARD_MULTIPLIER: f32 = 1.5;
pub const MIN_GUARD_TIME: f64 = 100.0;

//-----------------------------------------------------------------------------

pub const OVER_LIMIT_COUNT: u32 = 5;

//-----------------------------------------------------------------------------

pub const VBAT_DEVIDER_R1: f32 = 100_000.0;
pub const VBAT_DEVIDER_R2: f32 = 91_000.0;

//-----------------------------------------------------------------------------

pub const START_BLINK_COUNT: u32 = 5;
pub const START_BLINK_PERIOD_MS: u32 = 1000;

//-----------------------------------------------------------------------------

// включать счетчики за 2 периода измерения
pub const PREHEAT_MULTIPLIER: u32 = 2;

// Счетчик, отскрочки включения частотомера после включения питания
pub const F_CH_START_COUNT: u32 = 2;

// Экспериментальное значение, время с подачи питания до запуска генератора
// Канал температуры дает 100kHz как минимум перые 100 ms, берем запас
pub const GEN_COLD_STARTUP_TIME_MS: u32 = 200;
