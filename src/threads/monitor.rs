use defmt::Debug2Format;
use defmt::Display2Format;
use freertos_rust::{CriticalRegion, DurationTicks, FreeRtosSchedulerState, FreeRtosUtils};

/*
    DEBUG("FreeRTOS threadinfo:");
    // Avoid divide by zero errors.
    if (ulTotalRunTime > 0) {
      std::for_each(
          taskdata.cbegin(), taskdata.cend(), [ulTotalRunTime](auto &taskinfo) {
            auto ulStatsAsPercentage =
                taskinfo.ulRunTimeCounter / ulTotalRunTime;
            DEBUG("%20s: %c, P:%lu, Sf:%6u, %2lu%% (%lu t)",
                  taskinfo.pcTaskName,
                  _task_state_to_char(taskinfo.eCurrentState),
                  taskinfo.uxCurrentPriority, taskinfo.usStackHighWaterMark,
                  ulStatsAsPercentage, taskinfo.ulRunTimeCounter);
          });
    }

    DEBUG("");
    DEBUG("Current Heap Free Size: %u", xPortGetFreeHeapSize());
    DEBUG("Minimal Heap Free Size: %u", xPortGetMinimumEverFreeHeapSize());
    DEBUG("");
*/

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
