use core::borrow::BorrowMut;

use alloc::sync::Arc;

use freertos_rust::{CurrentTask, Duration, Mutex};

use nanopb_rs::{IStream, OStream, pb_decode::rx_context, pb_encode::tx_context};

use usb_device::class_prelude::UsbBus;

use usbd_serial::SerialPort;

static ERR_MSG: &str = "Failed to get serial port";

struct Reader<'a, B: UsbBus> {
    container: Arc<Mutex<SerialPort<'a, B>>>,
}

impl<'a, B: UsbBus> rx_context for Reader<'a, B> {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize, ()> {
        loop {
            match self.container.lock(Duration::infinite()) {
                Ok(mut serial) => match serial.borrow_mut().read(buff) {
                    Ok(count) => {
                        if count > 0 {
                            return Ok(count);
                        } else {
                            continue;
                        }
                    }
                    Err(_) => return Err(()),
                },
                Err(e) => panic!("{}: {:?}", ERR_MSG, e),
            }
        }
    }
}

struct Writer<'a, B: UsbBus> {
    container: Arc<Mutex<SerialPort<'a, B>>>,
}

impl<'a, B: UsbBus> tx_context for Writer<'a, B> {
    fn write(&mut self, src: &[u8]) -> Result<usize, ()> {
        loop {
            match self.container.lock(Duration::infinite()) {
                Ok(mut serial  ) => {
                    let mut write_offset = 0;
                    while write_offset < src.len() {
                        match serial.write(&src[write_offset..]) {
                            Ok(len) if len > 0 => {
                                write_offset += len;
                            }
                            _ => CurrentTask::delay(Duration::ms(1))
                        }
                    };
                    return Ok(src.len());
                },
                Err(e) => panic!("{}: {:?}", ERR_MSG, e),
            }
        }
    }
}

pub fn protobuf_server<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
) -> ! {
    let _is = IStream::from_callback(
        Reader {
            container: serial_container.clone(),
        },
        None,
    );

    let _os = OStream::from_callback(
        Writer {
            container: serial_container.clone(),
        },
        None,
    );

    loop {
        CurrentTask::delay(Duration::ms(10));
        /*
        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                defmt::info!("Serial> Ressived {} bytes", count);
                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        */

        /*
        core::mem::forget(
            freertos_rust::Task::current()
                .unwrap()
                .wait_for_notification(0, 0, Duration::ms(100)),
        );
        */
    }
}
