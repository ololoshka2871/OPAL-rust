/*
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
*/

pub fn monitord<D: freertos_rust::DurationTicks>(period: D) -> ! {
    use alloc::vec::Vec;
    use alloc::{format, string::ToString};
    use defmt::Display2Format;
    use freertos_rust::{CriticalRegion, FreeRtosSchedulerState, FreeRtosUtils};

    static HEADER: &str = "FreeRTOS threadinfo:\n   ID | Name       | State     | Priority | Stack left | CPU Abs.   |  %\n";

    let mut prev_statistics: Option<FreeRtosSchedulerState> = None;
    loop {
        let mut staticstics: FreeRtosSchedulerState;
        freertos_rust::CurrentTask::delay(period);

        {
            let _ = CriticalRegion::enter();
            staticstics = FreeRtosUtils::get_all_tasks(None);
        }

        staticstics
            .tasks
            .sort_by(|a, b| a.task_number.cmp(&b.task_number));

        let (diffs, runtime_from_last_call) = if let Some(prev_stat) = &prev_statistics {
            let total_run_time_diff = if prev_stat.total_run_time < staticstics.total_run_time {
                staticstics.total_run_time - prev_stat.total_run_time
            } else {
                freertos_rust::FreeRtosUnsignedLong::MAX - prev_stat.total_run_time
                    + staticstics.total_run_time
                    + 1
            };
            let run_time_counter_diffs = staticstics
                .tasks
                .iter_mut()
                .zip(prev_stat.tasks.iter())
                .map(|(s, ps)| {
                    if s.run_time_counter >= ps.run_time_counter {
                        s.run_time_counter - ps.run_time_counter
                    } else {
                        freertos_rust::FreeRtosUnsignedLong::MAX - ps.run_time_counter
                            + s.run_time_counter
                            + 1
                    }
                })
                .collect::<Vec<u32>>();
            (run_time_counter_diffs, total_run_time_diff)
        } else {
            let run_time_counter_diffs = staticstics
                .tasks
                .iter_mut()
                .map(|s| s.run_time_counter)
                .collect::<Vec<u32>>();

            (run_time_counter_diffs, staticstics.total_run_time)
        };

        let mut stat = staticstics
            .tasks
            .iter()
            .zip(diffs.iter())
            .fold(HEADER.to_string(), |mut acc, (task, diff)| {
                let s = format!(
                    "└─ {id: <2} | {name: <10} | {state: <9} | {priority: <8} | {stack: >10} | {cpu_abs: >10} | {cpu_rel: >4}\n",
                    id = task.task_number,
                    name = task.name,
                    state = alloc::format!("{:?}", task.task_state),
                    priority = task.current_priority.0,
                    stack = task.stack_high_water_mark,
                    cpu_abs = task.run_time_counter,
                    cpu_rel = if runtime_from_last_call > 0 && *diff <= runtime_from_last_call {
                        let p = ((diff * 100) / runtime_from_last_call) as u32;
                        let ps = if p == 0 && task.run_time_counter > 0 {
                            "<1".to_string()
                        } else {
                            p.to_string()
                        };
                        alloc::format!("{: >3}%", ps)
                    } else {
                        "-".to_string()
                    }
                );
                acc.push_str(s.as_str());
                acc
            });

        prev_statistics.replace(staticstics);

        //stat.push_str(format!("Total run time: {}", staticstics.total_run_time).as_str());

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
