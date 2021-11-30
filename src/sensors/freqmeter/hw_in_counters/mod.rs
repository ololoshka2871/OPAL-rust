pub trait InCounter<DMA, PIN> {
    /// just number of counter
    fn id(&self) -> u32;

    /// init timer
    fn init(&self);

    /// dma channel
    fn configure_dma(&self);

    /// input pin
    fn configure_gpio(&self);

    /*
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

    /// interrupt pending?
    fn is_irq_pending(&self, controller: &dyn IInterruptController) -> bool;
    */
}

#[cfg(feature = "stm32l433")]
mod in_counters_l433;
