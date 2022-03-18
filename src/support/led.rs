use core::convert::Infallible;

use alloc::boxed::Box;
use stm32l4xx_hal::{gpio::PinState, prelude::OutputPin};

struct MyPin(pub Box<dyn OutputPin<Error = Infallible>>);

unsafe impl Sync for MyPin {}

static mut LED: Option<MyPin> = None;

pub fn led_init<P>(pin: P)
where
    P: OutputPin<Error = Infallible> + 'static,
{
    unsafe {
        LED = Some(MyPin(Box::new(pin)));
    }
}

pub fn led_set(state: PinState) {
    if let Some(l) = unsafe { LED.as_mut() } {
        let _ = l.0.set_state(state);
    }
}
