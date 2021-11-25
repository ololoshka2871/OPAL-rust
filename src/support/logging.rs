//use core::sync::atomic::{AtomicUsize, Ordering};

/// global logger
use defmt_rtt as _;
use freertos_rust::FreeRtosUtils;
use panic_probe as _;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/*
static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
});
*/

defmt::timestamp!(
    "[{:?}]",
    crate::workmodes::common::Ticks(FreeRtosUtils::get_tick_count())
);
