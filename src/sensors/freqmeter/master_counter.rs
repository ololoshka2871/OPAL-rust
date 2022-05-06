#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::sync::Arc;
use stm32l4xx_hal::time::Hertz;

use crate::support::interrupt_controller::IInterruptController;

use super::hw_master::MasterCounterInfo;

pub struct MasterCounter {
    counter: &'static dyn MasterCounterInfo,
    interrupt_controller: Arc<dyn IInterruptController>,
    want_enable_counter: AtomicUsize,
    extander: u32,
    ref_freq: Hertz,
}

pub struct MasterTimerInfo {
    master: &'static mut MasterCounter,
    wanted_start: bool,
}

static mut MATSTER_COUNTER: Option<MasterCounter> = None;

impl MasterCounter {
    pub fn acquire() -> MasterTimerInfo {
        if unsafe { MATSTER_COUNTER.is_none() } {
            panic!("Master counter acquire before init");
        } else {
            MasterTimerInfo {
                master: unsafe { MATSTER_COUNTER.as_mut().unwrap_unchecked() },
                wanted_start: false,
            }
        }
    }

    pub fn init(ref_freq: Hertz, interrupt_controller: Arc<dyn IInterruptController>) {
        let master = super::hw_master::get_master_list()[0];

        master.init();

        master.set_interrupt_prio(
            &*interrupt_controller,
            crate::config::MASTER_COUNTER_INTERRUPT_PRIO,
        );

        unsafe {
            MATSTER_COUNTER = Some(MasterCounter {
                counter: master,
                interrupt_controller,
                want_enable_counter: AtomicUsize::new(0),
                extander: 0,
                ref_freq,
            })
        };
    }

    fn want_start(&mut self) {
        let v = self.want_enable_counter.fetch_add(1, Ordering::Relaxed);
        if v == 0 {
            self.counter.start();
            self.counter
                .enable_interrupt(self.interrupt_controller.as_ref(), true);
        }
    }

    fn want_stop(&mut self) {
        let v = self.want_enable_counter.fetch_sub(1, Ordering::Relaxed);
        if v == 1 {
            self.counter
                .enable_interrupt(self.interrupt_controller.as_ref(), false);
            self.counter.stop();
        }
    }

    #[inline]
    fn ovf_irq(&mut self, id: u32) {
        if id == self.counter.id() {
            cortex_m::interrupt::free(|_| {
                // Вся суть в том, чтобы обработчик прерывания рабочих
                // счетчиков ни как не мог быть вызван между актом инкремента
                // расширителя и сброса флага прерывания
                // тогда если в захваченном значении был флаг, но "сейчас" уже нет
                // прерывание было выполнено до конца, а если и был и есть - оно не выполнено
                // и надо давать +1 к значению расширителя
                self.counter
                    .clear_interrupt(self.interrupt_controller.as_ref());
                self.extander = self.extander.wrapping_add(1);
            });
        }
    }

    fn wrap_result_if_ovf_common(&self, mut value: u32) -> (u32, u32, bool) {
        let mut was_wraped = false;
        let ext = self.extander;
        if let Some(mask) = self.counter.uif_cpy_mask() {
            if (value & mask != 0)
                || self
                    .counter
                    .is_irq_pending(self.interrupt_controller.as_ref())
            {
                //ext = ext.wrapping_add(1);
                was_wraped = true;
            }
            value &= !mask;
        }
        (ext, value, was_wraped)
    }

    fn wrap_result_if_ovf(&self, value: u32) -> (u32, bool) {
        let (ext_wraped, value, was_wraped) = self.wrap_result_if_ovf_common(value);
        (value | (ext_wraped << 16) as u32, was_wraped)
    }

    fn wrap_result_if_ovf64(&self, value: u32) -> (u64, bool) {
        let (ext_wraped, value, was_wraped) = self.wrap_result_if_ovf_common(value);
        (value as u64 | ((ext_wraped as u64) << 16), was_wraped)
    }

    #[inline]
    fn cnt_addr(&self) -> usize {
        self.counter.cnt_addr()
    }

    pub fn f_ref(&self) -> Hertz {
        self.ref_freq
    }
}

impl MasterTimerInfo {
    #[inline]
    pub fn want_start(&mut self) {
        if !self.wanted_start {
            self.wanted_start = true;
            self.master.want_start()
        }
    }

    #[inline]
    pub fn want_stop(&mut self) {
        if self.wanted_start {
            self.wanted_start = false;
            self.master.want_stop()
        }
    }

    pub fn update_captured_value(&self, v: u32) -> (u32, bool) {
        self.master.wrap_result_if_ovf(v)
    }

    pub fn value(&self) -> (u32, bool) {
        let counter_value = self.master.counter.value();
        self.master.wrap_result_if_ovf(counter_value)
    }

    pub fn value64(&self) -> (u64, bool) {
        let counter_value = self.master.counter.value();
        self.master.wrap_result_if_ovf64(counter_value)
    }

    pub fn uptime_ms(&self) -> u64 {
        self.value64().0 / (self.f_ref().to_kHz() as u64)
    }

    #[inline]
    pub fn cnt_addr(&self) -> usize {
        self.master.cnt_addr()
    }

    #[inline]
    pub fn f_ref(&self) -> Hertz {
        self.master.f_ref()
    }
}

impl Drop for MasterTimerInfo {
    fn drop(&mut self) {
        self.want_stop();
    }
}

//-----------------------------------------------------------------------------

pub(crate) unsafe fn master_ovf(id: u32) {
    if MATSTER_COUNTER.is_some() {
        MATSTER_COUNTER.as_mut().unwrap_unchecked().ovf_irq(id)
    }
}
