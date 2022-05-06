//-----------------------------------------------------------------------------

#[cfg(feature = "clock-base-12Mhz")]
pub const XTAL_FREQ: u32 = 12_000_000;

#[cfg(feature = "clock-base-24Mhz")]
pub const XTAL_FREQ: u32 = 24_000_000;

//-----------------------------------------------------------------------------
// Это же число должно быть записано в src/configTemplate/FreeRTOSConfig.h через build.rs

#[cfg(feature = "recorder-power-save")]
pub const FREERTOS_CONFIG_FREQ: u32 = 3_000_000; // /4, /8

#[cfg(all(feature = "recorder-balanced", feature = "clock-base-12Mhz"))]
pub const FREERTOS_CONFIG_FREQ: u32 = 6_000_000; // /2

#[cfg(all(feature = "recorder-balanced", feature = "clock-base-24Mhz"))]
pub const FREERTOS_CONFIG_FREQ: u32 = 12_000_000; // /2

#[cfg(all(feature = "recorder-performance", feature = "clock-base-12Mhz"))]
pub const FREERTOS_CONFIG_FREQ: u32 = 12_000_000; // /1

#[cfg(all(feature = "recorder-performance", feature = "clock-base-24Mhz"))]
pub const FREERTOS_CONFIG_FREQ: u32 = 24_000_000; // /1

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

// Приоритеты, обльше -> лучше

/// pseudo-idle task prio
pub const IDLE_TASK_PRIO: u8 = 0;

/// usbd task prio
pub const USBD_TASK_PRIO: u8 = IDLE_TASK_PRIO + 3;

/// protobuf task prio
pub const PROTOBUF_TASK_PRIO: u8 = USBD_TASK_PRIO - 1; // иначе не работает

/// monitor task prio
pub const MONITOR_TASK_PRIO: u8 = IDLE_TASK_PRIO + 1;

/// sensor processor task prio
pub const SENS_PROC_TASK_PRIO: u8 = IDLE_TASK_PRIO + 8;

/// recorder controller task prio
pub const RECORDER_CTRL_PRIO: u8 = IDLE_TASK_PRIO + 4;

/// flash cleaner prio
pub const FLASH_CLEANER_PRIO: u8 = IDLE_TASK_PRIO + 2;

//-----------------------------------------------------------------------------

pub const INITIAL_FREQMETER_TARGET: u32 = 1;

//-----------------------------------------------------------------------------

pub const MINIMUM_ADAPTATION_INTERVAL: u32 = 50;
pub const MEASURE_TIME_TO_GUARD_MULTIPLIER: f32 = 1.5;
pub const MIN_GUARD_TIME: f64 = 100.0;

//-----------------------------------------------------------------------------

pub const OVER_LIMIT_COUNT: u32 = 5;

//-----------------------------------------------------------------------------

pub const VBAT_DEVIDER_R1: f32 = 270_000.0;
pub const VBAT_DEVIDER_R2: f32 = 91_000.0;

//-----------------------------------------------------------------------------

pub const START_BLINK_COUNT: u32 = 5;
pub const START_BLINK_PERIOD_MS: u32 = 500;

//-----------------------------------------------------------------------------

// включать счетчики за 2 периода измерения
pub const PREHEAT_MULTIPLIER: u32 = 2;

// Счетчик, отскрочки включения частотомера после включения питания
pub const F_CH_START_COUNT: u32 = 2;

// Экспериментальное значение, время с подачи питания до запуска генератора
// Канал температуры дает 100kHz как минимум перые 100 ms, берем запас
pub const GEN_COLD_STARTUP_TIME_MS: u32 = 200;

//-----------------------------------------------------------------------------

// Задержка перехода флешки с спящий режим при неактивности
pub const FLASH_AUTO_POWER_DOWN_MS: u32 = 10;
