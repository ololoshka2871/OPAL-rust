use stm32l4xx_hal::{
    dma::dma1,
    gpio::{Alternate, Floating, Input, AF1, PA0, PA8},
    stm32l4::stm32l4x2::{TIM1, TIM2},
};

use super::InCounter;

impl InCounter<dma1::C6, PA8<Alternate<AF1, Input<Floating>>>> for TIM1 {
    fn id(&self) -> u32 {
        todo!()
    }

    fn init(&self) {
        todo!()
    }

    fn configure_dma(&self) {
        todo!()
    }

    fn configure_gpio(&self) {
        todo!()
    }
}

impl InCounter<dma1::C2, PA0<Alternate<AF1, Input<Floating>>>> for TIM2 {
    fn id(&self) -> u32 {
        todo!()
    }

    fn init(&self) {
        todo!()
    }

    fn configure_dma(&self) {
        todo!()
    }

    fn configure_gpio(&self) {
        todo!()
    }
}
