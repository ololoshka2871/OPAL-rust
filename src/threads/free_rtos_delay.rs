use embedded_hal::blocking::delay::DelayUs;

pub struct FreeRtosDelay;

impl DelayUs<u32> for FreeRtosDelay {
    fn delay_us(&mut self, us: u32) {
        cortex_m::asm::delay(us * 10000);
    }
}
