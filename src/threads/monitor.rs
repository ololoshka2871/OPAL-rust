use defmt::Debug2Format;
use defmt::Display2Format;
use freertos_rust::{CriticalRegion, DurationTicks, FreeRtosSchedulerState, FreeRtosUtils};

pub fn monitord<D: DurationTicks>(period: D) -> ! {
    loop {
        let staticstics: FreeRtosSchedulerState;
        freertos_rust::CurrentTask::delay(period);

        {
            let _ = CriticalRegion::enter();
            staticstics = FreeRtosUtils::get_all_tasks(None);
        }

        defmt::info!("FreeRTOS threadinfo:");
        staticstics.tasks.iter().for_each(|task| {
            defmt::info!(
                "{}: {}, P:{}, Sf:{}",
                Display2Format(&task.name),
                Debug2Format(&task.task_state),
                task.current_priority.0,
                task.stack_high_water_mark
            );
        });
    }
}
