use alloc::boxed::Box;
use stm32l4xx_hal::{
    device::{tim1, tim2, RCC},
    dma::dma1,
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

trait Utils<T> {
    fn tim() -> &'static T;
    fn clk_enable();
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

        let tim = Self::tim();

        // pause
        tim.cr1.modify(|_, w| w.cen().clear_bit());

        // clear config
        tim.smcr.modify(|_, w| unsafe {
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

        // stm32l4xx_hal_tim.c:6569
        tim.ccer
            .modify(|_, w| w.cc1e().clear_bit().cc1p().clear_bit().cc1np().clear_bit());
        tim.ccmr1_input().modify(|_, w| w.ic1f().fck_int_n2());

        // configure clock input PA8 -> CH1
        // stm32l4xx_hal_tim.c:6786
        //tim.smcr.modify(|_, w| w.ts().itr1().sms().ext_clock_mode());
        tim.smcr.modify(|_, w| w.ts().itr1().sms().disabled());

        // initial state
        //tim.psc.write(|w| w.psc().bits(0)); // no prescaler
        tim.psc.write(|w| w.psc().bits(0xF000));

        // initial target
        tim.arr
            .write(|w| unsafe { w.bits(crate::config::INITIAL_FREQMETER_TARGET) });

        // Trigger DMA event to load the prescaler value to the clock
        tim.egr.write(|w| w.ug().set_bit());

        // configure dma event src
        // dma master -> buf
        dma.set_memory_address(unsafe { &TIM1_DMA_BUF as *const _ as u32 }, false);
        dma.set_peripheral_address(master_cnt_addr as u32, false);
        dma.set_transfer_length(core::mem::size_of::<u32>() as u16);

        // dma enable irq
        ic.unmask(IRQ::DMA1_CH2.into());

        // dma enable
        dma.listen(stm32l4xx_hal::dma::Event::TransferComplete);
        dma.start();
    }

    fn target() -> u32 {
        unsafe { (*TIM1::ptr()).arr.read().bits() }
    }
}

impl Enable for TIM1 {
    fn start(&mut self) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}

impl Utils<tim1::RegisterBlock> for TIM1 {
    fn tim() -> &'static tim1::RegisterBlock {
        unsafe { &*TIM1::ptr() }
    }

    fn clk_enable() {
        let enr = unsafe { &(*RCC::ptr()).apb2enr };
        let rstr = unsafe { &(*RCC::ptr()).apb2rstr };

        // enable and reset peripheral to a clean slate state
        enr.modify(|_, w| w.tim1en().set_bit());
        rstr.modify(|_, w| w.tim1rst().set_bit());
        rstr.modify(|_, w| w.tim1rst().clear_bit());
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

        let tim = Self::tim();

        // pause
        tim.cr1.modify(|_, w| w.cen().clear_bit());

        // clear config
        tim.smcr.modify(|_, w| unsafe {
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

        // stm32l4xx_hal_tim.c:6569
        tim.ccer
            .modify(|_, w| w.cc1e().clear_bit().cc1p().clear_bit().cc1np().clear_bit());
        tim.ccmr1_input().modify(|_, w| w.ic1f().fck_int_n2());

        // configure clock input PA8 -> CH1
        // stm32l4xx_hal_tim.c:6786
        tim.smcr.modify(|_, w| w.ts().itr1().sms().ext_clock_mode());

        // initial state
        tim.psc.write(|w| w.psc().bits(0)); // no prescaler

        // initial target
        tim.arr
            .write(|w| unsafe { w.bits(crate::config::INITIAL_FREQMETER_TARGET) });

        // Trigger DMA event to load the prescaler value to the clock
        tim.egr.write(|w| w.ug().set_bit());

        // configure dma event src
        // dma master -> buf
        dma.set_memory_address(unsafe { &TIM1_DMA_BUF as *const _ as u32 }, false);
        dma.set_peripheral_address(master_cnt_addr as u32, false);
        dma.set_transfer_length(core::mem::size_of::<u32>() as u16);

        // dma enable irq
        ic.unmask(IRQ::DMA1_CH2.into());

        // dma enable
        dma.listen(stm32l4xx_hal::dma::Event::TransferComplete);
        dma.start();
    }

    fn target() -> u32 {
        unsafe { (*TIM2::ptr()).arr.read().bits() }
    }
}

impl Enable for TIM2 {
    fn start(&mut self) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}

impl Utils<tim2::RegisterBlock> for TIM2 {
    fn tim() -> &'static tim2::RegisterBlock {
        unsafe { &*TIM2::ptr() }
    }

    fn clk_enable() {
        let enr = unsafe { &(*RCC::ptr()).apb1enr1 };
        let rstr = unsafe { &(*RCC::ptr()).apb1rstr1 };

        // enable and reset peripheral to a clean slate state
        enr.modify(|_, w| w.tim2en().set_bit());
        rstr.modify(|_, w| w.tim2rst().set_bit());
        rstr.modify(|_, w| w.tim2rst().clear_bit());
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
        TIM1_DMA_BUF,
        TIM1::target(),
        IRQ::DMA1_CH2.into(),
    );
}

#[interrupt]
unsafe fn DMA1_CH6() {
    call_dma_cb(
        &DMA1_CH6_CB,
        TIM2_DMA_BUF,
        TIM2::target(),
        IRQ::DMA1_CH6.into(),
    );
}
