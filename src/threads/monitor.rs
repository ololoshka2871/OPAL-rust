use alloc::string::ToString;
use defmt::Display2Format;
use freertos_rust::{CriticalRegion, DurationTicks, FreeRtosSchedulerState, FreeRtosUtils};

pub fn monitord<D: DurationTicks>(period: D) -> ! {
    static HEADER: &str = "FreeRTOS threadinfo:\n   ID | Name       | State     | Priority | Stack left | CPU Abs.   |  %\n";

    loop {
        let staticstics: FreeRtosSchedulerState;
        freertos_rust::CurrentTask::delay(period);

        {
            let _ = CriticalRegion::enter();
            staticstics = FreeRtosUtils::get_all_tasks(None);
        }

        let stat = staticstics
            .tasks
            .iter()
            .fold(HEADER.to_string(), |mut acc, task| {
                let s = alloc::format!(
                    "└─ {id: <2} | {name: <10} | {state: <9} | {priority: <8} | {stack: >10} | {cpu_abs: >10} | {cpu_rel: >4}\n",
                    id = task.task_number,
                    name = task.name,
                    state = alloc::format!("{:?}", task.task_state),
                    priority = task.current_priority.0,
                    stack = task.stack_high_water_mark,
                    cpu_abs = task.run_time_counter,
                    cpu_rel = if staticstics.total_run_time > 0 && task.run_time_counter <= staticstics.total_run_time {
                        let p = (((task.run_time_counter as u64) * 100) / staticstics.total_run_time as u64) as u32;
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

        defmt::info!("{}", Display2Format(&stat));
    }
}
