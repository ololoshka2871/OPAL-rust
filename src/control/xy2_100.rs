use core::{
    convert::Infallible,
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::sync::Arc;
use stm32l4xx_hal::{gpio::gpiod, interrupt, prelude::OutputPin};

use crate::support::interrupt_controller::IInterruptController;

// 1. Таймер T7 триггерит DMA2_CH5 которая копирует u16 из памяти в GPIOD
// 2. В буфере 40 u16 -> это 20 CLOCKов
// 3. Прерывание DMA считает переданное и как только накопится 20 останавливает процесс.
// 4. Если второй буфер готов к передаче буферы свапются и сразу начинается отправка
// 5. Загрузка новой команды всегда в теневой буфер.

const TX_POCKET_SIZE: usize = 20;

static mut OUTPUT_BUF_A: [u16; TX_POCKET_SIZE * 2] = [0; TX_POCKET_SIZE * 2];
static mut OUTPUT_BUF_B: [u16; TX_POCKET_SIZE * 2] = [0; TX_POCKET_SIZE * 2];

static mut TX_BUF: &mut [u16] = unsafe { &mut OUTPUT_BUF_A };
static mut BACK_BUF: &mut [u16] = unsafe { &mut OUTPUT_BUF_B };

static mut BACK_BUF_READY: AtomicBool = AtomicBool::new(false);

pub struct XY2_100<EN: OutputPin<Error = Infallible>> {
    timer7: stm32l4xx_hal::stm32l4::stm32l4x3::TIM7,
    _port: gpiod::Parts,
    dma: stm32l4xx_hal::dma::dma2::C5,

    clk_mask: u16,
    sync_mask: u16,
    pin_data_x_mask: u16,
    pin_data_y_mask: u16,

    enable_pin: Option<EN>,
}

impl<EN: OutputPin<Error = Infallible>> XY2_100<EN> {
    pub fn new(
        timer7: stm32l4xx_hal::stm32l4::stm32l4x3::TIM7,
        port: gpiod::Parts,
        dma2_ch6: stm32l4xx_hal::dma::dma2::C5,
        clk_pin_n: u16,
        sync_pin_n: u16,
        pin_data_x_pin_n: u16,
        pin_data_y_pin_n: u16,
        enable_pin: Option<EN>,
    ) -> Self {
        for n in [clk_pin_n, sync_pin_n, pin_data_x_pin_n, pin_data_y_pin_n] {
            unsafe {
                // stm32l4xx-hal/src/gpio/convert.rs
                const PUPDR: u32 = 0b00;
                const MODER: u32 = 0b01;
                const OTYPER: Option<u32> = Some(0b0);

                let offset = 2 * n;

                let ptr = &*stm32l4xx_hal::stm32l4::stm32l4x3::GPIOD::ptr();
                ptr.pupdr
                    .modify(|r, w| w.bits((r.bits() & !(0b11 << offset)) | (PUPDR << offset)));

                if let Some(otyper) = OTYPER {
                    ptr.otyper
                        .modify(|r, w| w.bits(r.bits() & !(0b1 << n) | (otyper << n)));
                }

                ptr.moder
                    .modify(|r, w| w.bits((r.bits() & !(0b11 << offset)) | (MODER << offset)));
            }
        }

        Self {
            timer7,
            _port: port,
            dma: dma2_ch6,

            clk_mask: 1u16 << clk_pin_n,
            sync_mask: 1u16 << sync_pin_n,
            pin_data_x_mask: 1u16 << pin_data_x_pin_n,
            pin_data_y_mask: 1u16 << pin_data_y_pin_n,

            enable_pin,
        }
    }

    pub fn begin(
        &mut self,
        ic: Arc<dyn IInterruptController>,
        tim_ref_clk: stm32l4xx_hal::time::Hertz,
    ) {
        // configure dma memory -> GPIO by tim7
        {
            use stm32l4xx_hal::stm32l4::stm32l4x3::Interrupt;

            // configure dma event src
            self.dma.stop();
            self.dma.set_memory_address(0u32, true); // not ready
            self.dma.set_peripheral_address(
                unsafe {
                    &((*stm32l4xx_hal::stm32l4::stm32l4x3::GPIOD::ptr()).odr) as *const _ as u32
                },
                false,
            );
            self.dma.set_transfer_length((TX_POCKET_SIZE * 2) as u16); // TX_POCKET_SIZE * 2 транзакций по таймера 16 -> 32

            unsafe {
                let dma_ptr = &*stm32l4xx_hal::device::DMA2::ptr();

                // Table 42. DMA2 requests for each channel [3:5]
                dma_ptr.cselr.modify(|_, w| w.c5s().map3());

                dma_ptr.ccr5.modify(|_, w| {
                    w.pl()
                        .very_high() // prio
                        .msize()
                        .bits16() // 16 bit
                        .psize()
                        .bits32() // 16 bit
                        .circ()
                        .clear_bit() // not circular
                        .dir()
                        .from_memory() // M -> p
                        .teie()
                        .enabled() // error irq - disable
                        .htie()
                        .disabled() // half transfer - disable
                        .tcie()
                        .enabled() // transfer compleead - enable
                });
            }

            // transfer complead interrupt
            ic.set_priority(Interrupt::DMA2_CH5.into(), crate::config::DMA_IRQ_PRIO);
            ic.unmask(Interrupt::DMA2_CH5.into());
        }

        // init timer
        {
            use crate::support::debug_mcu::DEBUG_MCU;
            use stm32l4xx_hal::device::RCC;

            let enr = unsafe { &(*RCC::ptr()).apb1enr1 };
            let rstr = unsafe { &(*RCC::ptr()).apb1rstr1 };

            let tim = &self.timer7;

            enr.modify(|_, w| w.tim7en().set_bit());
            rstr.modify(|_, w| w.tim7rst().set_bit());
            rstr.modify(|_, w| w.tim7rst().clear_bit());

            // no prescaler
            tim.psc.write(|w| unsafe { w.bits(0) });

            // autoreload
            tim.arr.write(|w| unsafe {
                w.bits(tim_ref_clk.to_Hz() / crate::config::GALVO_CLOCK_RATE - 1)
                // Calibrate for 2MHz CLK TICK rate
            });

            // Trigger an update event to load the prescaler value to the clock
            tim.egr.write(|w| w.ug().set_bit());

            // enable overflow interrupt
            tim.dier.write(|w| w.uie().bit(true));

            // reset DMA request
            tim.sr.modify(|_, w| w.uif().clear_bit());

            // DMA request on overflow
            tim.dier.modify(|_, w| w.ude().set_bit());

            // start counter
            //tim.cnt.write(|w| unsafe { w.bits(0) });
            //tim.cr1.modify(|_, w| w.cen().set_bit());

            // __HAL_DBGMCU_FREEZE_TIM7() -> SET_BIT(DBGMCU->APB1FZR1, DBGMCU_APB1FZR1_DBG_TIM7_STOP[5])
            unsafe {
                (*DEBUG_MCU)
                    .apb1fzr1
                    .set((*DEBUG_MCU).apb1fzr1.get() | (1 << 5));
            }
        }
    }

    pub fn set_pos(&mut self, x: u16, y: u16) {
        let data_x = Self::build_msg(x);
        let data_y = Self::build_msg(y);

        unsafe {
            BACK_BUF.iter_mut().enumerate().for_each(|(i, r)| {
                let bit_n = i / 2;

                *r = self.sync_mask; // sync == 1 by default

                // clk
                if i & 1 == 0 {
                    *r |= self.clk_mask;
                }
                // sync == 0 only last bit
                if bit_n == TX_POCKET_SIZE - 1 {
                    *r &= !self.sync_mask;
                }

                // data
                let chk_mask = 1u32 << (TX_POCKET_SIZE - bit_n - 1);
                if data_x & chk_mask != 0 {
                    *r |= self.pin_data_x_mask
                }
                if data_y & chk_mask != 0 {
                    *r |= self.pin_data_y_mask
                }
            });

            BACK_BUF_READY.store(true, Ordering::SeqCst);

            start_tx();
        }
    }

    pub fn parity(v: u32) -> u32 {
        v.count_ones() % 2
    }

    fn build_msg(data: u16) -> u32 {
        // ... [0 0 1 <data16> <parity>] = 20 bit total
        let mut res = (0b001u32 << 17) | ((data as u32) << 1);
        res |= Self::parity(res);
        res
    }

    pub(crate) fn enable(&mut self) {
        if let Some(en_pin) = self.enable_pin.as_mut() {
            let _ = en_pin.set_state(crate::config::GALVO_EN_ACTIVE_LVL.into());
        }
    }

    pub(crate) fn disable(&mut self) {
        if let Some(en_pin) = self.enable_pin.as_mut() {
            let _ = en_pin.set_state((!crate::config::GALVO_EN_ACTIVE_LVL).into());
        }
    }
}

unsafe fn start_tx() {
    if !BACK_BUF_READY.load(Ordering::SeqCst) {
        return;
    }

    let tim7 = &*stm32l4xx_hal::device::TIM7::ptr();
    let dma = &*stm32l4xx_hal::device::DMA2::ptr();

    if tim7.cr1.read().cen().bit_is_set() {
        return; /* not ready */
    }

    BACK_BUF_READY.store(false, Ordering::SeqCst); // back buffer not ready
    core::mem::swap(&mut TX_BUF, &mut BACK_BUF); // swap buffers

    // stop dma2_ch5
    dma.ifcr.write(|w| w.cgif5().set_bit());
    dma.ccr5.modify(|_, w| w.en().clear_bit());

    // set font buffer adress
    dma.cmar5.write(|w| w.ma().bits(TX_BUF.as_ptr() as u32));

    // transfer length
    dma.cndtr5
        .write(|w| w.ndt().bits((TX_POCKET_SIZE * 2) as u16));

    // enable dma2_ch5
    dma.ccr5.modify(|_, w| w.en().set_bit());

    // enable timer
    tim7.cnt.write(|w| w.bits(0));
    tim7.cr1.modify(|_, w| w.cen().set_bit());
}

// вся передача завершена
#[interrupt]
unsafe fn DMA2_CH5() {
    let dma = &*stm32l4xx_hal::device::DMA2::ptr();
    let tim7 = &*stm32l4xx_hal::device::TIM7::ptr();

    // clear event
    dma.ifcr.write(|w| w.cgif5().set_bit());

    tim7.cr1.modify(|_, w| w.cen().clear_bit());
    if BACK_BUF_READY.load(Ordering::SeqCst) {
        start_tx();
    }
}
