use freertos_rust::{FreeRtosTaskHandle, FreeRtosUBaseType};

extern "C" {
    fn uxTaskPriorityGet(pxTask: FreeRtosTaskHandle) -> FreeRtosUBaseType;
    fn vTaskPrioritySet(pxTask: FreeRtosTaskHandle, uxNewPriority: FreeRtosUBaseType);
}

pub fn mast_yield() {
    unsafe {
        let task = freertos_rust::freertos_rs_get_current_task();
        let current_prio = uxTaskPriorityGet(task);
        vTaskPrioritySet(task, crate::config::IDLE_TASK_PRIO.into());

        freertos_rust::freertos_rs_isr_yield();

        vTaskPrioritySet(task, current_prio);
    }
}
