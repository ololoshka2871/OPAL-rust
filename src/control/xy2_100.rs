use core::convert::Infallible;
use core::ptr::null;

use alloc::boxed::Box;
use alloc::sync::Arc;
use stm32l4xx_hal::interrupt;

use stm32l4xx_hal::prelude::{OutputPin, PinState};
use vcell::VolatileCell;

static mut XY2_CTX: Option<xy2_100_ctx> = None;

struct xy2_100_ctx {
    timer7: stm32l4xx_hal::stm32l4::stm32l4x3::TIM7,

    pin_clk: Box<dyn OutputPin<Error = Infallible>>,
    pin_sync: Box<dyn OutputPin<Error = Infallible>>,
    pin_data_x: Box<dyn OutputPin<Error = Infallible>>,
    pin_data_y: Box<dyn OutputPin<Error = Infallible>>,

    a_new_packet_x: VolatileCell<[bool; 20]>,
    a_new_packet_y: VolatileCell<[bool; 20]>,
    b_new_packet_x: VolatileCell<[bool; 20]>,
    b_new_packet_y: VolatileCell<[bool; 20]>,

    active_packet_x: VolatileCell<*const bool>,
    active_packet_y: VolatileCell<*const bool>,

    flag: bool,
}

pub struct xy2_100;

impl xy2_100 {
    pub fn new(
        timer7: stm32l4xx_hal::stm32l4::stm32l4x3::TIM7,
        pin_clk: Box<dyn OutputPin<Error = Infallible>>,
        pin_sync: Box<dyn OutputPin<Error = Infallible>>,
        pin_data_x: Box<dyn OutputPin<Error = Infallible>>,
        pin_data_y: Box<dyn OutputPin<Error = Infallible>>,
    ) -> Self {
        unsafe {
            let res = xy2_100_ctx {
                timer7,

                pin_clk,
                pin_sync,
                pin_data_x,
                pin_data_y,

                a_new_packet_x: VolatileCell::new([false; 20]),
                a_new_packet_y: VolatileCell::new([false; 20]),
                b_new_packet_x: VolatileCell::new([false; 20]),
                b_new_packet_y: VolatileCell::new([false; 20]),

                active_packet_x: VolatileCell::new(null::<bool>()),
                active_packet_y: VolatileCell::new(null::<bool>()),

                flag: false,
            };

            res.active_packet_x.set(res.a_new_packet_x.get().as_ptr());
            res.active_packet_y.set(res.a_new_packet_y.get().as_ptr());

            XY2_CTX = Some(res);
        }

        Self {}
    }

    pub fn begin(
        &mut self,
        interrupt_controller: Arc<dyn crate::support::interrupt_controller::IInterruptController>,
        tim_ref_clk: stm32l4xx_hal::time::Hertz,
    ) {
        use stm32l4xx_hal::stm32l4::stm32l4x3::Interrupt;

        // init timer
        {
            use crate::support::debug_mcu::DEBUG_MCU;
            use stm32l4xx_hal::device::RCC;

            let enr = unsafe { &(*RCC::ptr()).apb1enr1 };
            let rstr = unsafe { &(*RCC::ptr()).apb1rstr1 };
            if let Some(xy2_ctx) = unsafe { XY2_CTX.as_ref() } {
                let tim = &xy2_ctx.timer7;

                enr.modify(|_, w| w.tim7en().set_bit());
                rstr.modify(|_, w| w.tim7rst().set_bit());
                rstr.modify(|_, w| w.tim7rst().clear_bit());

                // no prescaler
                tim.psc.write(|w| unsafe { w.bits(0) });

                // autoreload
                tim.arr.write(|w| unsafe {
                    w.bits(tim_ref_clk.to_Hz() / 100_000) // Calibrate for 2MHz
                });

                // Trigger an update event to load the prescaler value to the clock
                tim.egr.write(|w| w.ug().set_bit());

                // enable overflow interrupt
                tim.dier.write(|w| w.uie().bit(true));

                // start counter
                tim.cr1.modify(|_, w| w.cen().set_bit());

                // __HAL_DBGMCU_FREEZE_TIM7() -> SET_BIT(DBGMCU->APB1FZR1, DBGMCU_APB1FZR1_DBG_TIM7_STOP[5])
                unsafe {
                    (*DEBUG_MCU)
                        .apb1fzr1
                        .set((*DEBUG_MCU).apb1fzr1.get() | (1 << 5));
                }
            }
        }

        // enable timer interrupt
        interrupt_controller.set_priority(
            Interrupt::TIM7.into(),
            crate::config::GALVO_INTERFACE_TICK_PRIO,
        );
        interrupt_controller.unmask(Interrupt::TIM7.into());
    }

    pub fn set_pos(&self, x: u16, y: u16) {}

    pub fn parity(v: u16) -> u8 {
        0
    }

    fn build_msg(data: u16) -> u32 {
        0
    }
}

#[interrupt]
unsafe fn TIM7() {
    use stm32l4xx_hal::stm32l4::stm32l4x3::Interrupt;

    if let Some(xy2_ctx) = XY2_CTX.as_mut() {
        xy2_ctx.pin_clk.set_high();

        // clear interrupt flag
        xy2_ctx.timer7.sr.write(|w| w.uif().clear_bit());
    }

    //cortex_m::peripheral::NVIC::mask(Interrupt::TIM7);
    cortex_m::peripheral::NVIC::unpend(Interrupt::TIM7);
}
