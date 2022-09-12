use alloc::sync::Arc;
use stm32f1xx_hal::gpio::{Output, PushPull, PB3, PB4, PB5, PB6};

use crate::support::interrupt_controller::IInterruptController;

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
        todo!()
    }

    fn set_pos(&mut self, x: u16, y: u16) {
        todo!()
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
        Self {
            timer,
            dma,
            port_addr: unsafe { &(*port_ptr).bsrr as *const _ as usize },
            outputs,
        }
    }
}
