#![allow(dead_code)]

use core::convert::Infallible;

use alloc::boxed::Box;
use stm32l4xx_hal::prelude::OutputPin;

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

/// can call from any place
/// --Rust--
/// extern "C" {
///     pub fn led_set(state: u8);
/// }
///
/// --or (C/C++)--
///
/// extern void led_set(uint8_t state);
/// ...
/// unsafe { led_set(0); }
#[no_mangle]
pub extern "C" fn led_set(state: u8) {
    if let Some(l) = unsafe { LED.as_mut() } {
        let _ = if state == 0 {
            l.0.set_high() // led OFF
        } else {
            l.0.set_low() // led ON
        };
    }
}
