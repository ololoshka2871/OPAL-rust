use crate::support::interrupt_controller::{IInterruptController, Interrupt};

pub trait OnCycleFinished: Sync {
    fn cycle_finished(
        &self,
        /*event: TimerEvent,*/ captured: u32,
        target: u32,
        irq: Interrupt,
    );
}

pub trait InCounter<DMA, PIN> {
    /// init timer
    fn configure<CB: 'static + OnCycleFinished>(
        &mut self,
        master_cnt_addr: usize,
        dma: &mut DMA,
        input: PIN, // сам пин не используется, но нужен для выведения типа и поглащается
        ic: &dyn IInterruptController,
        dma_complead: CB,
    );

    fn target32(&self) -> u32;
    fn reset(&mut self);
    fn set_target32(&mut self, target: u32);

    fn cold_start(&mut self);
    fn stop(&mut self) -> bool;
}

#[cfg(feature = "stm32l433")]
mod in_counters_l433;
