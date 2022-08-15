pub fn monitord<D: freertos_rust::DurationTicks>(period: D) -> ! {
    use alloc::{format, string::ToString};
    use defmt::Display2Format;
    use freertos_rust::{CriticalRegion, FreeRtosSchedulerState, FreeRtosUtils};

    static HEADER: &str =
        "FreeRTOS threadinfo:\n   ID | Name       | State     | Priority | Stack left\n";

    loop {
        let staticstics: FreeRtosSchedulerState;
        freertos_rust::CurrentTask::delay(period);

        {
            let _ = CriticalRegion::enter();
            staticstics = FreeRtosUtils::get_all_tasks(None);
        }

        let mut stat = staticstics
            .tasks
            .iter()
            .fold(HEADER.to_string(), |mut acc, task| {
                let s = format!(
                    "└─ {id: <2} | {name: <10} | {state: <9} | {priority: <8} | {stack: >10}\n",
                    id = task.task_number,
                    name = task.name,
                    state = alloc::format!("{:?}", task.task_state),
                    priority = task.current_priority.0,
                    stack = task.stack_high_water_mark,
                );
                acc.push_str(s.as_str());
                acc
            });

        #[cfg(feature = "monitor-heap")]
        {
            extern "C" {
                fn xPortGetFreeHeapSize() -> usize;
                fn xPortGetMinimumEverFreeHeapSize() -> usize;
            }

            unsafe {
                stat.push_str(
                    format!(
                        "\nHeap statistics: Free {} bytes, Min: {} bytes",
                        xPortGetFreeHeapSize(),
                        xPortGetMinimumEverFreeHeapSize()
                    )
                    .as_str(),
                );
            }
        }

        defmt::info!("{}", Display2Format(&stat));
    }
}
