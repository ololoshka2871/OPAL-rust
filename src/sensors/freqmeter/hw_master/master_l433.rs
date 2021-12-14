use stm32l4xx_hal::{
    device::{tim6, RCC},
    interrupt,
    stm32l4::stm32l4x3::Interrupt as IRQ,
};

use crate::support::interrupt_controller::{IInterruptController, Interrupt};

use super::MasterCounterInfo;

struct Tim6_7MasterCounter {
    id: u8,
}

impl Tim6_7MasterCounter {
    fn tim(&self) -> &'static tim6::RegisterBlock {
        let addr = match self.id {
            6 => 0x4000_1000_usize, // stm32l4-0.12.1/src/stm32l4x2/mod.rs:1032
            7 => 0x4000_1400_usize, // stm32l4-0.12.1/src/stm32l4x2/mod.rs:1053
            _ => panic!(),
        };

        unsafe { &*(addr as *const tim6::RegisterBlock) }
    }

    fn interrupt_n(&self) -> Interrupt {
        match self.id {
            6 => IRQ::TIM6_DAC.into(),
            7 => IRQ::TIM7.into(),
            _ => panic!(),
        }
    }
}

impl MasterCounterInfo for Tim6_7MasterCounter {
    fn id(&self) -> u32 {
        self.id as u32
    }

    // stm32l4xx-hal-0.6.0/src/timer.rs
    fn init(&self) {
        let enr = unsafe { &(*RCC::ptr()).apb1enr1 };
        let rstr = unsafe { &(*RCC::ptr()).apb1rstr1 };

        match self.id {
            6 => {
                enr.modify(|_, w| w.tim6en().set_bit());
                rstr.modify(|_, w| w.tim6rst().set_bit());
                rstr.modify(|_, w| w.tim6rst().clear_bit());
            }
            7 => {
                enr.modify(|_, w| w.tim7en().set_bit());
                rstr.modify(|_, w| w.tim7rst().set_bit());
                rstr.modify(|_, w| w.tim7rst().clear_bit());
            }
            _ => panic!(),
        }
    }

    fn set_interrupt_prio(&self, controller: &dyn IInterruptController, prio: u8) {
        controller.set_priority(self.interrupt_n(), prio);
    }

    fn start(&self) {
        // pause
        self.stop();

        let tim = self.tim();

        // no prescaler
        tim.psc.write(|w| unsafe { w.bits(0) });

        // autoreload
        tim.arr.write(|w| unsafe { w.bits(u16::MAX as u32) });

        // Trigger an update event to load the prescaler value to the clock
        tim.egr.write(|w| w.ug().set_bit());

        // enable UIF_CPY
        tim.cr1
            .modify(|r, w| unsafe { w.bits(r.bits() | 1u32 << 11) });

        // start counter
        tim.cr1.modify(|_, w| w.cen().set_bit());
    }

    fn stop(&self) {
        self.tim().cr1.modify(|_, w| w.cen().clear_bit());
    }

    fn clear_interrupt(&self, controller: &dyn IInterruptController) {
        self.tim().sr.write(|w| w.uif().clear_bit());
        controller.unpend(self.interrupt_n());
    }

    fn enable_interrupt(&self, controller: &dyn IInterruptController, enable: bool) {
        let irq = self.interrupt_n();
        if enable {
            controller.unmask(irq);
        } else {
            controller.mask(irq);
        }

        self.tim().dier.write(|w| w.uie().bit(enable));
    }

    fn value(&self) -> u32 {
        self.tim().cnt.read().bits() & (u16::MAX as u32)
    }

    fn cnt_addr(&self) -> usize {
        &self.tim().cnt as *const _ as usize
    }

    fn uif_cpy_mask(&self) -> Option<u32> {
        Some(1u32 << 31)
    }

    fn is_irq_pending(&self, controller: &dyn IInterruptController) -> bool {
        controller.is_pending(self.interrupt_n())
    }
}

pub(crate) static MASTER_LIST: [&dyn MasterCounterInfo; 2] = [
    &Tim6_7MasterCounter { id: 6 },
    &Tim6_7MasterCounter { id: 7 },
];

#[interrupt]
unsafe fn TIM6_DAC() {
    crate::sensors::freqmeter::master_counter::master_ovf(6);
}

#[interrupt]
unsafe fn TIM7() {
    crate::sensors::freqmeter::master_counter::master_ovf(7);
}
