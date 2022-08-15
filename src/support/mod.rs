pub mod free_rtos_error_ext;
pub mod hex_slice;
pub mod interrupt_controller;
pub mod led;
pub mod len_in_u64_aligned;
pub mod log_anywhere;
pub mod logging;
pub mod timer_period;

mod freertos_hooks;
mod new_freertos_timer;
mod new_global_mutex;

#[cfg(feature = "stm32l433")]
mod interrupt_controller_l433;

#[cfg(feature = "stm32l433")]
pub use interrupt_controller_l433::InterruptController;

#[cfg(debug_assertions)]
pub mod debug_mcu;

pub use new_freertos_timer::new_freertos_timer;
pub use new_global_mutex::new_global_mutex;
