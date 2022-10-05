use core::sync::atomic::Ordering;

use alloc::sync::Arc;
use stm32f1xx_hal::gpio::{Output, PinExt, PushPull, PB3, PB4, PB5, PB6};

use crate::support::interrupt_controller::IInterruptController;

use super::{build_msg, BACK_BUF, BACK_BUF_READY, TX_BUF, TX_POCKET_SIZE};

impl super::XY2_100Interface
    for super::XY2_100<
        stm32f1xx_hal::device::TIM2,
        stm32f1xx_hal::dma::dma1::C2,
        (
            PB3<Output<PushPull>>,
            PB4<Output<PushPull>>,
            PB5<Output<PushPull>>,
            PB6<Output<PushPull>>,
        ),
    >
{
    fn begin<IC: IInterruptController>(
        &mut self,
        ic: Arc<IC>,
        tim_ref_clk: stm32f1xx_hal::time::Hertz,
    ) {
        // configure dma memory -> GPIO by tim2_up
        {
            use stm32f1xx_hal::device::Interrupt;

            // configure dma event src
            self.dma.stop();
            self.dma.set_memory_address(0u32, true); // not ready
            self.dma.set_peripheral_address(self.port_addr, false);
            self.dma.set_transfer_length(TX_POCKET_SIZE * 2); // TX_POCKET_SIZE * 2 транзакций по таймера 16 -> 32

            unsafe {
                (*stm32f1xx_hal::device::DMA1::ptr()).ch2.cr.modify(|_, w| {
                    w.pl()
                        .very_high() // prio
                        .msize()
                        .bits16() // 16 bit
                        .psize()
                        .bits32() // 32 bit
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
            ic.set_priority(Interrupt::DMA1_CHANNEL2.into(), crate::config::DMA_IRQ_PRIO);
            ic.unmask(Interrupt::DMA1_CHANNEL2.into());
        }

        // init timer
        {
            use crate::support::debug_mcu::DEBUG_MCU;
            use stm32f1xx_hal::device::RCC;

            let enr = unsafe { &(*RCC::ptr()).apb1enr };
            let rstr = unsafe { &(*RCC::ptr()).apb1rstr };

            let tim = &self.timer;

            enr.modify(|_, w| w.tim2en().set_bit());
            rstr.modify(|_, w| w.tim2rst().set_bit());
            core::sync::atomic::compiler_fence(Ordering::SeqCst);
            rstr.modify(|_, w| w.tim2rst().clear_bit());

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

            // SET_BIT(DBGMCU->APB1FZR1, DBGMCU_APB1FZR1_DBG_TIM2_STOP[0])
            unsafe {
                (*DEBUG_MCU)
                    .apb1fzr1
                    .set((*DEBUG_MCU).apb1fzr1.get() | (1 << 0));
            }
        }
    }

    fn set_pos(&mut self, x: u16, y: u16) {
        let data_x = build_msg(x);
        let data_y = build_msg(y);

        unsafe {
            BACK_BUF.iter_mut().enumerate().for_each(|(i, r)| {
                let (clk_mask, sync_mask, pin_data_x_mask, pin_data_y_mask) = (
                    1 << self.outputs.0.pin_id(),
                    1 << self.outputs.1.pin_id(),
                    1 << self.outputs.2.pin_id(),
                    1 << self.outputs.3.pin_id(),
                );

                let bit_n = i / 2;

                *r = sync_mask; // sync == 1 by default

                // clk
                if i & 1 == 0 {
                    *r |= clk_mask;
                }
                // sync == 0 only last bit
                if bit_n == TX_POCKET_SIZE - 1 {
                    *r &= !sync_mask;
                }

                // data
                let chk_mask = 1u32 << (TX_POCKET_SIZE - bit_n - 1);
                if data_x & chk_mask != 0 {
                    *r |= pin_data_x_mask
                }
                if data_y & chk_mask != 0 {
                    *r |= pin_data_y_mask
                }
            });

            BACK_BUF_READY.store(true, Ordering::SeqCst);

            Self::start_tx();
        }
    }
}

impl
    super::XY2_100<
        stm32f1xx_hal::device::TIM2,
        stm32f1xx_hal::dma::dma1::C2,
        (
            PB3<Output<PushPull>>,
            PB4<Output<PushPull>>,
            PB5<Output<PushPull>>,
            PB6<Output<PushPull>>,
        ),
    >
{
    pub fn new(
        timer: stm32f1xx_hal::device::TIM2,
        dma: stm32f1xx_hal::dma::dma1::C2,
        port_ptr: *const stm32f1xx_hal::device::gpioa::RegisterBlock,
        outputs: (
            PB3<Output<PushPull>>,
            PB4<Output<PushPull>>,
            PB5<Output<PushPull>>,
            PB6<Output<PushPull>>,
        ),
    ) -> Self {
        unsafe { super::DMA1_CH2_IT = Some(Self::dma_event) };

        Self {
            timer,
            dma,
            port_addr: unsafe { &(*port_ptr).odr as *const _ as u32 },
            outputs,
        }
    }

    unsafe fn dma_event() {
        let dma = &*stm32f1xx_hal::device::DMA1::ptr();
        let tim2 = &*stm32f1xx_hal::device::TIM2::ptr();

        // clear event
        dma.ifcr.write(|w| w.cgif2().set_bit());

        tim2.cr1.modify(|_, w| w.cen().clear_bit());
        if BACK_BUF_READY.load(Ordering::SeqCst) {
            Self::start_tx();
        }
    }

    unsafe fn start_tx() {
        if !BACK_BUF_READY.load(Ordering::SeqCst) {
            return;
        }

        let tim = &*stm32f1xx_hal::device::TIM2::ptr();
        let dma = &*stm32f1xx_hal::device::DMA1::ptr();

        if tim.cr1.read().cen().bit_is_set() {
            return; /* not ready */
        }

        BACK_BUF_READY.store(false, Ordering::SeqCst); // back buffer not ready
        core::mem::swap(&mut TX_BUF, &mut BACK_BUF); // swap buffers

        // stop dma1_ch2
        dma.ifcr.write(|w| w.cgif2().set_bit()); // reset interrupt flag
        dma.ch2.cr.modify(|_, w| w.en().clear_bit());

        // set font buffer adress
        dma.ch2.mar.write(|w| w.ma().bits(TX_BUF.as_ptr() as u32));

        // transfer length
        dma.ch2
            .ndtr
            .write(|w| w.ndt().bits((TX_POCKET_SIZE * 2) as u16));

        // enable dma1_ch2
        dma.ch2.cr.modify(|_, w| w.en().set_bit());

        // enable timer
        tim.cnt.write(|w| w.bits(0));
        tim.cr1.modify(|_, w| w.cen().set_bit());
    }
}
