use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::sync::Arc;

use crate::support::interrupt_controller::IInterruptController;

use super::hw_master::MasterCounterInfo;

pub struct MasterCounter {
    counter: &'static dyn MasterCounterInfo,
    interrupt_controller: Arc<dyn IInterruptController>,
    want_enable_counter: AtomicUsize,
    extander: u16,
}

pub struct MasterTimerInfo {
    master: &'static mut MasterCounter,
    wanted_start: bool,
}

static mut MATSTER_COUNTER: Option<MasterCounter> = None;

impl MasterCounter {
    pub fn allocate() -> Result<MasterTimerInfo, ()> {
        if unsafe { MATSTER_COUNTER.is_none() } {
            Err(())
        } else {
            Ok(MasterTimerInfo {
                master: unsafe { MATSTER_COUNTER.as_mut().unwrap() },
                wanted_start: false,
            })
        }
    }

    pub fn init(interrupt_controller: Arc<dyn IInterruptController>) {
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
            })
        };
    }

    fn want_start(&mut self) {
        let v = self.want_enable_counter.fetch_add(1, Ordering::Relaxed);
        if v == 0 {
            self.counter.start();
            self.counter
                .enable_interrupt(&*self.interrupt_controller, true);
        }
    }

    fn want_stop(&mut self) {
        let v = self.want_enable_counter.fetch_sub(1, Ordering::Relaxed);
        if v == 1 {
            self.counter
                .enable_interrupt(&*self.interrupt_controller, false);
            self.counter.stop();
        }
    }

    #[inline]
    fn ovf_irq(&mut self, id: u32) {
        if id == self.counter.id() {
            self.extander = self.extander.wrapping_add(1);
            self.counter.clear_interrupt(&*self.interrupt_controller);
        }
    }

    fn wrap_result_if_ovf(&self, mut value: u32) -> (u32, bool) {
        let mut was_wraped = false;
        let mut ext = self.extander as u32;
        if let Some(mask) = self.counter.uif_cpy_mask() {
            if value & mask == mask {
                value &= !mask;
                ext = ext.wrapping_add(1);
                was_wraped = true;
            }
        }

        (value | (ext << 16), was_wraped)
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
}

impl Drop for MasterTimerInfo {
    fn drop(&mut self) {
        self.want_stop();
    }
}

//-----------------------------------------------------------------------------

pub(crate) unsafe fn master_ovf(id: u32) {
    if MATSTER_COUNTER.is_some() {
        MATSTER_COUNTER.as_mut().unwrap().ovf_irq(id)
    }
}
