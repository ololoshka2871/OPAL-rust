use core::borrow::BorrowMut;

use alloc::sync::Arc;
use freertos_rust::{CurrentTask, Duration, FreeRtosError, Mutex};
use nanopb_rs::pb_decode::rx_context;
use usb_device::{class_prelude::UsbBus, UsbError};
use usbd_serial::SerialPort;

pub struct Reader<'a, B: UsbBus> {
    pub container: Arc<Mutex<SerialPort<'a, B>>>,
}

impl<'a, B: UsbBus> rx_context for Reader<'a, B> {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize, ()> {
        fn block_thread() {
            CurrentTask::delay(Duration::ms(1));
        }

        loop {
            match self.container.lock(Duration::infinite()) {
                Ok(mut serial) => {
                    let ser = serial.borrow_mut();
                    match ser.read(buff) {
                        Ok(count) => {
                            if count > 0 {
                                //defmt::trace!("Serial: {} bytes ressived", count);
                                return Ok(count);
                            } else {
                                block_thread()
                            }
                        }
                        Err(UsbError::WouldBlock) => block_thread(),
                        Err(_) => return Err(()),
                    }
                }
                Err(e) => panic_log(e),
            }
        }
    }
}

fn panic_log(e: FreeRtosError) -> ! {
    static ERR_MSG: &str = "Failed to get serial port";

    defmt::panic!("{}: {:?}", ERR_MSG, defmt::Debug2Format(&e));
}
