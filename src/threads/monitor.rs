use defmt::Format;
use freertos_rust::{CriticalRegion, DurationTicks, FreeRtosSchedulerState, FreeRtosUtils};

struct FreeRtosSchedulerStateWraper(FreeRtosSchedulerState);

impl Format for FreeRtosSchedulerStateWraper {
    fn format(&self, fmt: defmt::Formatter) {
        fmt.inner.display(&self.0);
    }
}

pub fn monitord<D: DurationTicks>(period: D) -> ! {
    let mut staticstics: FreeRtosSchedulerStateWraper;
    let mut a = 0;
    loop {
        freertos_rust::CurrentTask::delay(period);
        /*/
        {
            let _ = CriticalRegion::enter();
            staticstics = FreeRtosSchedulerStateWraper(FreeRtosUtils::get_all_tasks(None));
        }
        defmt::info!("{}", staticstics);
        */
        defmt::info!("monitord={}", a);
        a += 1;
    }
}
