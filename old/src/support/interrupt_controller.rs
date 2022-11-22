use cortex_m::interrupt::InterruptNumber;

#[derive(Clone, Copy)]
pub struct Interrupt(pub u16);

unsafe impl InterruptNumber for Interrupt {
    fn number(self) -> u16 {
        self.0
    }
}

pub trait IInterruptController: Sync + Send {
    fn set_priority(&self, interrupt: Interrupt, prio: u8);
    fn unmask(&self, interrupt: Interrupt);
    fn mask(&self, interrupt: Interrupt);
    fn unpend(&self, interrupt: Interrupt);
    fn is_pending(&self, interrupt: Interrupt) -> bool;
}
