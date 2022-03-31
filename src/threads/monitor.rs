use alloc::{sync::Arc, vec::Vec};
use freertos_rust::Mutex;

use crate::workmodes::output_storage::OutputStorage;

pub fn monitord<D: freertos_rust::DurationTicks>(
    period: D,
    _output: Arc<Mutex<OutputStorage>>,
) -> ! {
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

        #[cfg(feature = "monitor-output")]
        {
            use crate::threads::sensor_processor::FChannel;
            use freertos_rust::Duration;

            // Output: | P   | T (*C) | TCPU (*C) | Vbat (v)
            // Output: |    0.100 |    1.000 |   32.890 |    3.360

            static M_HEADER: &str = "\nOutput: | P         | T (*C)    | TCPU (*C) | Vbat (v)\n";
            stat.push_str(M_HEADER);
            let _ = _output.lock(Duration::infinite()).map(|out| {
                stat.push_str(
                    format!(
                        "Output: | {P:<9.3} | {T:<9.3} | {TCPU:<9.3} | {VBAT:<8.3}\n",
                        P = out.values[FChannel::Pressure as usize].unwrap_or(f64::NAN),
                        T = out.values[FChannel::Temperature as usize].unwrap_or(f64::NAN),
                        TCPU = out.t_cpu,
                        VBAT = out.vbat,
                    )
                    .as_str(),
                );
            });
        }

        defmt::info!("{}", Display2Format(&stat));
    }
}
