use core::{
    panic,
    sync::atomic::{self, Ordering},
};

use alloc::boxed::Box;
use stm32l4xx_hal::{
    device::{tim1, tim2, DMA1, RCC},
    dma::{dma1, Event},
    gpio::{Alternate, PushPull, PA0, PA8},
    interrupt,
    stm32l4::stm32l4x3::{Interrupt as IRQ, TIM1, TIM2},
};
use vcell::VolatileCell;

use crate::support::interrupt_controller::{IInterruptController, Interrupt};

use super::{InCounter, OnCycleFinished, TimerEvent};

pub type DmaCb = Box<dyn OnCycleFinished>;

trait Utils<T, DMA> {
    fn clk_enable();
    fn select_dma_channel(dma: &mut DMA);

    fn target() -> u32;
    fn prescaler() -> u32;

    fn set_prescaler(psc: u32);
    fn set_reload(reload: u32);

    fn opm() -> bool;
    fn set_opm();

    #[cfg(debug_assertions)]
    fn configure_debug_freeze();
}

impl InCounter<dma1::C6, PA8<Alternate<PushPull, 1>>> for TIM1 {
    fn configure<CB: 'static + OnCycleFinished>(
        &mut self,
        master_cnt_addr: usize,
        dma: &mut dma1::C6,
        _input: PA8<Alternate<PushPull, 1>>,
        ic: &dyn IInterruptController,
        dma_complead: CB,
    ) {
        unsafe {
            set_cb(&mut DMA1_CH6_CB, dma_complead);
        }

        Self::clk_enable();

        // pause
        self.stop();

        // clear config
        self.smcr.modify(|_, w| unsafe {
            w.sms()
                .disabled()
                .ts()
                .bits(0b000)
                .etf()
                .bits(0b000)
                .etps()
                .div1()
                .ece()
                .clear_bit()
                .etp()
                .clear_bit()
        });

        self.cr1.modify(|_, w| {
            w.ckd()
                .div1()
                .cms()
                .edge_aligned()
                .dir()
                .up()
                .opm()
                .clear_bit()
                .urs()
                .set_bit() // update event generation disable
                .udis()
                .clear_bit()
        });

        // stm32l4xx_hal_tim.c:6569
        self.ccer.modify(|_, w| {
            w.cc1e()
                .clear_bit()
                // raising edge
                .cc1p()
                .clear_bit()
                .cc1np()
                .clear_bit()
        });
        // filter
        self.ccmr1_input().modify(|_, w| w.ic1f().fck_int_n8());

        // configure clock input PA0 -> CH1
        // stm32l4xx_hal_tim.c:6786
        self.smcr
            .modify(|_, w| w.ts().ti1fp1().sms().ext_clock_mode());

        //self.smcr.modify(|_, w| w.ts().itr1().sms().disabled());

        // initial state
        self.set_target32(crate::config::INITIAL_FREQMETER_TARGET);

        // reset DMA request
        self.sr.modify(|_, w| w.uif().clear_bit());

        // DMA request on overflow
        self.dier.modify(|_, w| w.ude().set_bit());

        atomic::compiler_fence(Ordering::SeqCst);

        // configure dma event src
        // dma master -> buf
        dma.stop();
        dma.set_memory_address(unsafe { TIM1_DMA_BUF.as_ptr() as u32 }, false);
        dma.set_peripheral_address(master_cnt_addr as u32, false);
        dma.set_transfer_length(1); // 1 транзакция 32 -> 32
        Self::select_dma_channel(dma);

        // в dma .ccrX() приватное, поэтому руками
        unsafe {
            (*DMA1::ptr()).ccr6.modify(|_, w| {
                w.pl()
                    .very_high() // prio
                    .msize()
                    .bits32() // 32 bit
                    .psize()
                    .bits32() // 32 bit
                    .circ()
                    .set_bit() // circular mode
                    .dir()
                    .from_peripheral() // p -> M
                    .teie()
                    .enabled() // error irq - disable
                    .htie()
                    .disabled() // half transfer - disable
            });
        }

        // dma enable irq
        ic.set_priority(IRQ::DMA1_CH6.into(), crate::config::DMA_IRQ_PRIO);
        ic.unmask(IRQ::DMA1_CH6.into());

        // dma enable
        dma.listen(Event::TransferComplete);
        dma.start();

        #[cfg(debug_assertions)]
        Self::configure_debug_freeze();
    }

    fn reset(&mut self) {
        self.egr.write(|w| w.ug().set_bit());
    }

    fn target32(&self) -> u32 {
        as_target32(Self::prescaler(), Self::target())
    }

    fn set_target32(&mut self, target: u32) {
        if self.cr1.read().cen().bit_is_set() {
            panic!("Attempt to change target of running Timer 1");
        }

        let (psc, reload) = transform_target32(target);

        Self::set_prescaler(psc);
        Self::set_reload(reload);

        self.reset();
    }

    fn cold_start(&mut self) {
        self.cr1
            .modify(|_, w| w.cen().clear_bit().opm().clear_bit());
        self.cnt
            .write(|w| unsafe { w.bits(self.arr.read().bits() - 1) });
        self.cr1.modify(|_, w| w.cen().set_bit());
    }

    fn stop(&mut self) -> bool {
        let res = self.cr1.read().cen().bit_is_set();
        self.cr1.modify(|_, w| w.cen().clear_bit());

        res
    }
}

impl Utils<tim1::RegisterBlock, dma1::C6> for TIM1 {
    fn clk_enable() {
        let apb2enr = unsafe { &(*RCC::ptr()).apb2enr };
        let apb2rstr = unsafe { &(*RCC::ptr()).apb2rstr };

        // enable and reset peripheral to a clean slate state
        apb2enr.modify(|_, w| w.tim1en().set_bit());
        apb2rstr.modify(|_, w| w.tim1rst().set_bit());
        apb2rstr.modify(|_, w| w.tim1rst().clear_bit());
    }

    fn select_dma_channel(_dma: &mut dma1::C6) {
        // stm32l433.pdf:p.299 -> TIM1_UP
        unsafe {
            (*DMA1::ptr()).cselr.modify(|_, w| w.c6s().map7());
        }
    }

    #[cfg(debug_assertions)]
    fn configure_debug_freeze() {
        use crate::support::debug_mcu::DEBUG_MCU;

        // __HAL_DBGMCU_FREEZE_TIM1() implementation
        unsafe {
            (*DEBUG_MCU)
                .apb2fz
                .set((*DEBUG_MCU).apb2fz.get() | (1 << 11));
        }
    }

    fn set_prescaler(psc: u32) {
        unsafe { (*Self::ptr()).psc.write(|w| w.bits(psc)) }
    }

    fn set_reload(reload: u32) {
        unsafe { (*Self::ptr()).arr.write(|w| w.bits(reload)) }
    }

    fn target() -> u32 {
        unsafe { (*Self::ptr()).arr.read().bits() }
    }

    fn prescaler() -> u32 {
        unsafe { (*Self::ptr()).psc.read().bits() }
    }

    fn opm() -> bool {
        unsafe { (*Self::ptr()).cr1.read().opm().bit_is_set() }
    }

    fn set_opm() {
        unsafe { (*Self::ptr()).cr1.modify(|_, w| w.opm().set_bit()) }
    }
}

impl InCounter<dma1::C2, PA0<Alternate<PushPull, 1>>> for TIM2 {
    fn configure<CB: 'static + OnCycleFinished>(
        &mut self,
        master_cnt_addr: usize,
        dma: &mut dma1::C2,
        _input: PA0<Alternate<PushPull, 1>>,
        ic: &dyn IInterruptController,
        dma_complead: CB,
    ) {
        unsafe {
            set_cb(&mut DMA1_CH2_CB, dma_complead);
        }

        Self::clk_enable();

        // pause
        self.stop();

        // clear config
        self.smcr.modify(|_, w| unsafe {
            w.sms()
                .disabled()
                .ts()
                .bits(0b000)
                .etf()
                .bits(0b000)
                .etps()
                .div1()
                .ece()
                .clear_bit()
                .etp()
                .clear_bit()
        });

        self.cr1.modify(|_, w| {
            w.ckd()
                .div1()
                .cms()
                .edge_aligned()
                .dir()
                .up()
                .opm()
                .clear_bit()
                .urs()
                .set_bit() // update event generation disable
                .udis()
                .clear_bit()
        });

        // stm32l4xx_hal_tim.c:6569
        self.ccer.modify(|_, w| {
            w.cc1e()
                .clear_bit()
                // raising edge
                .cc1p()
                .clear_bit()
                .cc1np()
                .clear_bit()
        });
        // filter
        self.ccmr1_input().modify(|_, w| w.ic1f().fck_int_n8());

        // configure clock input PA0 -> CH1
        // stm32l4xx_hal_tim.c:6786
        self.smcr
            .modify(|_, w| w.ts().ti1fp1().sms().ext_clock_mode());

        //self.smcr.modify(|_, w| w.ts().itr1().sms().disabled());

        // initial state
        self.set_target32(crate::config::INITIAL_FREQMETER_TARGET);

        // reset DMA request
        self.sr.modify(|_, w| w.uif().clear_bit());

        // DMA request on overflow
        self.dier.modify(|_, w| w.ude().set_bit());

        atomic::compiler_fence(Ordering::SeqCst);

        // configure dma event src
        // dma master -> buf
        dma.stop();
        dma.set_memory_address(unsafe { TIM2_DMA_BUF.as_ptr() as u32 }, false);
        dma.set_peripheral_address(master_cnt_addr as u32, false);
        dma.set_transfer_length(1); // 1 транзакция 32 -> 32
        Self::select_dma_channel(dma);

        // в dma .ccrX() приватное, поэтому руками
        unsafe {
            (*DMA1::ptr()).ccr2.modify(|_, w| {
                w.pl()
                    .very_high() // prio
                    .msize()
                    .bits32() // 32 bit
                    .psize()
                    .bits32() // 32 bit
                    .circ()
                    .set_bit() // circular mode
                    .dir()
                    .from_peripheral() // p -> M
                    .teie()
                    .enabled() // error irq - enable
                    .htie()
                    .disabled() // half transfer - disable
            });
        }

        // dma enable irq
        ic.set_priority(IRQ::DMA1_CH2.into(), crate::config::DMA_IRQ_PRIO);
        ic.unmask(IRQ::DMA1_CH2.into());

        // dma enable
        dma.listen(Event::TransferComplete);
        dma.start();

        #[cfg(debug_assertions)]
        Self::configure_debug_freeze();
    }

    fn reset(&mut self) {
        self.egr.write(|w| w.ug().set_bit());
    }

    fn target32(&self) -> u32 {
        as_target32(Self::prescaler(), Self::target())
    }

    fn set_target32(&mut self, target: u32) {
        if self.cr1.read().cen().bit_is_set() {
            panic!("Attempt to change target of running Timer 2");
        }

        let (psc, reload) = transform_target32(target);

        Self::set_prescaler(psc);
        Self::set_reload(reload);

        self.reset();
    }

    fn cold_start(&mut self) {
        self.cr1
            .modify(|_, w| w.cen().clear_bit().opm().clear_bit());
        self.cnt
            .write(|w| unsafe { w.bits(self.arr.read().bits() - 1) });
        self.cr1.modify(|_, w| w.cen().set_bit());
    }

    fn stop(&mut self) -> bool {
        let res = self.cr1.read().cen().bit_is_set();
        self.cr1.modify(|_, w| w.cen().clear_bit());

        res
    }
}

impl Utils<tim2::RegisterBlock, dma1::C2> for TIM2 {
    fn clk_enable() {
        let apb1enr1 = unsafe { &(*RCC::ptr()).apb1enr1 };
        let apb1rstr1 = unsafe { &(*RCC::ptr()).apb1rstr1 };

        // enable and reset peripheral to a clean slate state
        apb1enr1.modify(|_, w| w.tim2en().set_bit());
        apb1rstr1.modify(|_, w| w.tim2rst().set_bit());
        apb1rstr1.modify(|_, w| w.tim2rst().clear_bit());
    }

    fn select_dma_channel(_dma: &mut dma1::C2) {
        // stm32l433.pdf:p.299 -> TIM1_UP
        unsafe { (*DMA1::ptr()).cselr.modify(|_, w| w.c2s().map4()) }
    }

    #[cfg(debug_assertions)]
    fn configure_debug_freeze() {
        use crate::support::debug_mcu::DEBUG_MCU;

        // __HAL_DBGMCU_FREEZE_TIM1() implementation
        unsafe {
            (*DEBUG_MCU)
                .apb1fzr1
                .set((*DEBUG_MCU).apb1fzr1.get() | (1 << 0));
        }
    }

    fn target() -> u32 {
        unsafe { (*Self::ptr()).arr.read().bits() }
    }

    fn prescaler() -> u32 {
        unsafe { (*Self::ptr()).psc.read().bits() }
    }

    fn set_prescaler(psc: u32) {
        unsafe { (*Self::ptr()).psc.write(|w| w.bits(psc)) }
    }

    fn set_reload(reload: u32) {
        unsafe { (*Self::ptr()).arr.write(|w| w.bits(reload)) }
    }

    fn opm() -> bool {
        unsafe { (*Self::ptr()).cr1.read().opm().bit_is_set() }
    }

    fn set_opm() {
        unsafe { (*Self::ptr()).cr1.modify(|_, w| w.opm().set_bit()) }
    }
}

static mut TIM1_DMA_BUF: VolatileCell<u32> = VolatileCell::new(0);
static mut TIM2_DMA_BUF: VolatileCell<u32> = VolatileCell::new(0);

static mut DMA1_CH2_CB: Option<DmaCb> = None;
static mut DMA1_CH6_CB: Option<DmaCb> = None;

fn set_cb<CB: 'static + OnCycleFinished>(cb: &mut Option<DmaCb>, f: CB) {
    *cb = Some(Box::new(f));
}

fn as_target32(prescaler: u32, reload: u32) -> u32 {
    (prescaler + 1) * reload
}

fn transform_target32(mut target: u32) -> (u32, u32) {
    if target < 3 {
        target = 3;
    }
    for prescaler in 1..u16::MAX as u32 {
        let reload = target / prescaler;
        if reload < u16::MAX as u32 {
            return (prescaler - 1, reload);
        }
    }
    panic!("Can't find prescaler for target: {}", target);
}

unsafe fn call_dma_cb(
    cb: &Option<DmaCb>,
    opm_val: bool,
    captured: u32,
    prescaler: u32,
    reload: u32,
    irq: Interrupt,
) {
    if let Some(f) = cb {
        // reload | prescaler -> 32 bit target
        // f.cycle_finished(captured, as_target32(prescaler, reload), irq);
        f.cycle_finished(
            if opm_val {
                TimerEvent::Stop
            } else {
                TimerEvent::Start
            },
            captured,
            as_target32(prescaler, reload),
            irq,
        );
    }
}

#[interrupt]
unsafe fn DMA1_CH2() {
    let dma = &*DMA1::ptr();
    if dma.isr.read().teif2().bits() {
        panic!(
            "DMA1_CH2: Transferr error: 0x{:08X} -> 0x{:08X} count {}",
            dma.cpar2.read().bits(),
            dma.cmar2.read().bits(),
            dma.cndtr2.read().bits()
        );
    }

    // reset interrupt flag
    dma.ifcr.write(|w| w.cgif2().set_bit());

    let opm = TIM2::opm();

    #[cfg(feature = "freqmeter-start-stop")]
    if !opm {
        TIM2::set_opm()
    }

    call_dma_cb(
        &DMA1_CH2_CB,
        opm,
        TIM2_DMA_BUF.get(),
        TIM2::prescaler(),
        TIM2::target(),
        IRQ::DMA1_CH2.into(),
    );
}

#[interrupt]
unsafe fn DMA1_CH6() {
    let dma = &*DMA1::ptr();
    if dma.isr.read().teif6().bits() {
        panic!(
            "DMA1_CH2: Transferr error: 0x{:08X} -> 0x{:08X} count {}",
            dma.cpar6.read().bits(),
            dma.cmar6.read().bits(),
            dma.cndtr6.read().bits()
        );
    }

    // reset interrupt flag
    dma.ifcr.write(|w| w.cgif6().set_bit());

    let opm = TIM1::opm();
    #[cfg(feature = "freqmeter-start-stop")]
    if !opm {
        TIM1::set_opm()
    }

    call_dma_cb(
        &DMA1_CH6_CB,
        opm,
        TIM1_DMA_BUF.get(),
        TIM1::prescaler(),
        TIM1::target(),
        IRQ::DMA1_CH6.into(),
    );
}
