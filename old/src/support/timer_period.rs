use freertos_rust::Timer;

pub trait TimerExt {
    fn period(&self) -> u32;
}

impl TimerExt for Timer {
    fn period(&self) -> u32 {
        // pub struct Timer {
        //  handle: FreeRtosTimerHandle,
        //  detached: bool,
        // }

        extern "C" {
            fn xTimerGetPeriod(
                id: freertos_rust::FreeRtosVoidPtr,
            ) -> freertos_rust::FreeRtosTickType;
        }

        unsafe {
            let timer_handle = *(self as *const Timer as *const freertos_rust::FreeRtosTimerHandle);
            xTimerGetPeriod(timer_handle)
        }
    }
}
