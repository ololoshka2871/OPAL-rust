/// global logger
use defmt_rtt as _;

/*
use core::sync::atomic::{AtomicUsize, Ordering};

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
});
*/

use freertos_rust::FreeRtosUtils;

defmt::timestamp!(
    "[{:?}T]",
    crate::support::ticks_format::Ticks(FreeRtosUtils::get_tick_count())
);
