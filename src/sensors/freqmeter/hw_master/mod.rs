pub(crate) trait MasterCounterInfo: Sync + Send {
    /// just number of counter
    fn id(&self) -> u32;

    /// init timer
    fn init(&self);

    /// interrupt priority
    fn set_interrupt_prio(&self, controller: &dyn IInterruptController, prio: u8);

    /// start counting
    fn start(&self);

    /// stop counting
    fn stop(&self);

    /// Enable or disable interrupt
    fn enable_interrupt(&self, controller: &dyn IInterruptController, enable: bool);

    /// clear interrupt flag
    fn clear_interrupt(&self, controller: &dyn IInterruptController);

    /// current value of counter register
    fn value(&self) -> u32;

    /// DMA source address
    fn cnt_addr(&self) -> usize;
}

#[cfg(feature = "stm32l433")]
mod master_l433;
#[cfg(feature = "stm32l433")]
use master_l433::MASTER_LIST;

use crate::support::interrupt_controller::IInterruptController;

pub(crate) fn get_master_list() -> &'static [&'static dyn MasterCounterInfo] {
    &MASTER_LIST
}
