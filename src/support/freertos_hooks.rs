#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes_definitions)]

use core::alloc::Layout;

use cortex_m_rt::{exception, ExceptionFrame};
use freertos_rust::{FreeRtosCharPtr, FreeRtosTaskHandle};

#[exception]
unsafe fn DefaultHandler(irqn: i16) {
    // custom default handler
    // irqn is negative for Cortex-M exceptions
    // irqn is positive for device specific (line IRQ)
    defmt::error!("Unregistred irq: {}", irqn);
    panic!();
}

#[exception]
unsafe fn HardFault(_ef: &ExceptionFrame) -> ! {
    defmt::error!("HardFault");
    panic!()
}

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    defmt::panic!("Heap allocation failed");
}

#[no_mangle]
fn vApplicationStackOverflowHook(_pxTask: FreeRtosTaskHandle, pcTaskName: FreeRtosCharPtr) {
    defmt::panic!("Thread {} stack overflow detected!", pcTaskName);
}

#[no_mangle]
fn vApplicationMallocFailedHook() {
    defmt::panic!("malloc() failed");
}

// libcore panic -> this function
// need if lto = false
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(
    _fmt: ::core::fmt::Arguments,
    file: &'static str,
    line: u32,
) -> ! {
    cortex_m::asm::udf();
}
