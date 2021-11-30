use crate::sensors::freqmeter::InCounter;

pub struct SensorPerith<TIM1, DMA1, TIM2, DMA2, PIN1, PIN2>
// Суть в том, что мы напишем КОНКРЕТНУЮ имплементацию InCounter<DMA> для
// конкретного счетчика рандомная пара не соберется.
where
    TIM1: InCounter<DMA1, PIN1>,
    TIM2: InCounter<DMA2, PIN2>,
{
    pub timer1: TIM1,
    pub timer1_dma_ch: DMA1,
    pub timer1_pin: PIN1,
    pub timer2: TIM2,
    pub timer2_dma_ch: DMA2,
    pub timer2_pin: PIN2,
}

pub fn sensor_processor<TIM1, DMA1, TIM2, DMA2, PIN1, PIN2>(
    perith: SensorPerith<TIM1, DMA1, TIM2, DMA2, PIN1, PIN2>,
) -> !
where
    TIM1: InCounter<DMA1, PIN1>,
    TIM2: InCounter<DMA2, PIN2>,
{
    let master_counter =
        crate::sensors::freqmeter::master_counter::MasterCounter::allocate().unwrap();

    /*
        perith.timer1.init();
        perith.timer2.init();
    */
    loop {
        unsafe {
            freertos_rust::freertos_rs_isr_yield();
        }
    }
}
