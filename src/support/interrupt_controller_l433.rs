use super::interrupt_controller::{self, Interrupt};
use cortex_m::{interrupt::InterruptNumber, peripheral::NVIC};
use stm32l4xx_hal::stm32l4::stm32l4x3::Interrupt as IRQ;

pub struct InterruptController(cortex_m::peripheral::NVIC);

unsafe impl Sync for InterruptController {}
unsafe impl Send for InterruptController {}

impl InterruptController {
    pub fn new(nvic: cortex_m::peripheral::NVIC) -> Self {
        Self(nvic)
    }
}

impl interrupt_controller::IInterruptController for InterruptController {
    fn set_priority(&self, interrupt: interrupt_controller::Interrupt, prio: u8) {
        unsafe {
            // Это костыль, но лучше решения я не знаю
            // Суть в том, что обертка контроллера прерываний запихана в Arc
            // и расшарена между потоками, но при этом содержимое Arc не должно меняться
            // а если надо менять надо юзать Mutex, который нельзя использовать, поскольку
            // unpend() вызывается в контексте прерывания
            let c: *mut cortex_m::peripheral::NVIC = &self.0 as *const _ as *mut _;
            (*c).set_priority(interrupt, prio)
        };
    }

    fn unmask(&self, interrupt: interrupt_controller::Interrupt) {
        unsafe { NVIC::unmask(interrupt) };
    }

    fn mask(&self, interrupt: interrupt_controller::Interrupt) {
        NVIC::mask(interrupt);
    }

    fn unpend(&self, interrupt: interrupt_controller::Interrupt) {
        NVIC::unpend(interrupt);
    }

    fn is_pending(&self, interrupt: Interrupt) -> bool {
        NVIC::is_pending(interrupt)
    }
}

impl Into<Interrupt> for IRQ {
    fn into(self) -> Interrupt {
        Interrupt { 0: self.number() }
    }
}
