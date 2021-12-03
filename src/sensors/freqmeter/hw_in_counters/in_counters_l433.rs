use core::{
    ptr,
    sync::atomic::{self, AtomicU32, Ordering},
};

use alloc::boxed::Box;
use stm32l4xx_hal::{
    device::{tim1, tim2, DMA1, RCC},
    dma::{dma1, Event},
    gpio::{Alternate, Floating, Input, AF1, PA0, PA8},
    interrupt,
    stm32l4::stm32l4x2::{Interrupt as IRQ, TIM1, TIM2},
};

use crate::{
    sensors::Enable,
    support::interrupt_controller::{IInterruptController, Interrupt},
};

use super::{InCounter, OnCycleFinished};

pub type DmaCb = Box<dyn OnCycleFinished>;

trait Utils<T, DMA> {
    fn clk_enable();
    fn select_dma_channel(dma: &mut DMA);
}

impl InCounter<dma1::C6, PA8<Alternate<AF1, Input<Floating>>>> for TIM1 {
    fn configure<CB: 'static + OnCycleFinished>(
        &mut self,
        master_cnt_addr: usize,
        dma: &mut dma1::C6,
        _input: PA8<Alternate<AF1, Input<Floating>>>,
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
                .clear_bit() // update event generation
                .udis()
                .clear_bit()
        });

        // stm32l4xx_hal_tim.c:6569
        self.ccer
            .modify(|_, w| w.cc1e().clear_bit().cc1p().clear_bit().cc1np().clear_bit());
        self.ccmr1_input().modify(|_, w| w.ic1f().fck_int_n2());

        // configure clock input PA8 -> CH1
        // stm32l4xx_hal_tim.c:6786
        //tim.smcr.modify(|_, w| w.ts().itr1().sms().ext_clock_mode()); // TODO clock src
        self.smcr.modify(|_, w| w.ts().itr1().sms().disabled());

        // initial state
        //tim.psc.write(|w| w.psc().bits(0)); // TODO: no prescaler
        self.psc.write(|w| w.psc().bits(0xF000));

        // initial target
        self.arr
            .write(|w| unsafe { w.bits(crate::config::INITIAL_FREQMETER_TARGET) });

        // reset DMA request
        self.sr.modify(|_, w| w.uif().clear_bit());

        // DMA request on overflow
        self.dier.modify(|_, w| w.ude().set_bit());

        atomic::compiler_fence(Ordering::SeqCst);

        // configure dma event src
        // dma master -> buf
        dma.stop();
        dma.set_peripheral_address(unsafe { &TIM1_DMA_BUF as *const _ as u32 }, false);
        dma.set_memory_address(master_cnt_addr as u32, false);
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
                    .clear_bit() // not circular
                    .dir()
                    .from_peripheral() // p -> M
                    .teie()
                    .disabled() // error irq - disable
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
    }

    fn target() -> u32 {
        unsafe { (*TIM1::ptr()).arr.read().bits() }
    }
}

impl Enable for TIM1 {
    fn start(&mut self) {
        self.cr1.modify(|_, w| w.cen().set_bit());
    }

    fn stop(&mut self) {
        self.cr1.modify(|_, w| w.cen().clear_bit());
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
        let dma_reg = unsafe { &*DMA1::ptr() };

        // stm32l433.pdf:p.299 -> TIM1_UP
        dma_reg.cselr.modify(|_, w| w.c6s().map7());
    }
}

impl InCounter<dma1::C2, PA0<Alternate<AF1, Input<Floating>>>> for TIM2 {
    fn configure<CB: 'static + OnCycleFinished>(
        &mut self,
        master_cnt_addr: usize,
        dma: &mut dma1::C2,
        _input: PA0<Alternate<AF1, Input<Floating>>>,
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
                .clear_bit() // update event generation
                .udis()
                .clear_bit()
        });

        // stm32l4xx_hal_tim.c:6569
        self.ccer
            .modify(|_, w| w.cc1e().clear_bit().cc1p().clear_bit().cc1np().clear_bit());
        self.ccmr1_input().modify(|_, w| w.ic1f().fck_int_n2());

        // configure clock input PA0 -> CH1
        // stm32l4xx_hal_tim.c:6786
        //tim.smcr.modify(|_, w| w.ts().itr1().sms().ext_clock_mode()); // TODO clock src
        self.smcr.modify(|_, w| w.ts().itr1().sms().disabled());

        // initial state
        //tim.psc.write(|w| w.psc().bits(0)); // TODO: no prescaler
        self.psc.write(|w| w.psc().bits(0xF000));

        // initial target
        self.arr
            .write(|w| unsafe { w.bits(crate::config::INITIAL_FREQMETER_TARGET) });

        // reset DMA request
        self.sr.modify(|_, w| w.uif().clear_bit());

        // DMA request on overflow
        self.dier.modify(|_, w| w.ude().set_bit());

        atomic::compiler_fence(Ordering::SeqCst);

        // configure dma event src
        // dma master -> buf
        dma.stop();
        dma.set_peripheral_address(unsafe { &TIM2_DMA_BUF as *const _ as u32 }, false);
        dma.set_memory_address(master_cnt_addr as u32, false);
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
                    .clear_bit() // not circular
                    .dir()
                    .from_peripheral() // p -> M
                    .teie()
                    .disabled() // error irq - disable
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
    }

    fn target() -> u32 {
        unsafe { (*TIM2::ptr()).arr.read().bits() }
    }
}

impl Enable for TIM2 {
    fn start(&mut self) {
        self.cr1.modify(|_, w| w.cen().set_bit());
    }

    fn stop(&mut self) {
        self.cr1.modify(|_, w| w.cen().clear_bit());
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
        let dma_reg = unsafe { &*DMA1::ptr() };

        // stm32l433.pdf:p.299 -> TIM1_UP
        dma_reg.cselr.modify(|_, w| w.c2s().map4());
    }
}

static mut TIM1_DMA_BUF: u32 = 0;
static mut TIM2_DMA_BUF: u32 = 0;

static mut DMA1_CH2_CB: Option<DmaCb> = None;
static mut DMA1_CH6_CB: Option<DmaCb> = None;

fn set_cb<CB: 'static + OnCycleFinished>(cb: &mut Option<DmaCb>, f: CB) {
    *cb = Some(Box::new(f));
}

unsafe fn call_dma_cb(cb: &Option<DmaCb>, captured: u32, target: u32, irq: Interrupt) {
    if let Some(f) = cb {
        f.cycle_finished(captured, target, irq);
    }
}

#[interrupt]
unsafe fn DMA1_CH2() {
    call_dma_cb(
        &DMA1_CH2_CB,
        ptr::read_volatile(&TIM1_DMA_BUF as *const _),
        TIM1::target(),
        IRQ::DMA1_CH2.into(),
    );
}

#[interrupt]
unsafe fn DMA1_CH6() {
    call_dma_cb(
        &DMA1_CH6_CB,
        ptr::read_volatile(&TIM2_DMA_BUF as *const _),
        TIM2::target(),
        IRQ::DMA1_CH6.into(),
    );
}
