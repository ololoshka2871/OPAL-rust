use core::borrow::BorrowMut;

use alloc::sync::Arc;

use freertos_rust::{CurrentTask, Duration, FreeRtosError, Mutex};

use nanopb_rs::{pb_decode::rx_context, pb_encode::tx_context, Error, IStream, OStream};

use usb_device::{class_prelude::UsbBus, UsbError};

use usbd_serial::SerialPort;

use crate::protobuf;

static ERR_MSG: &str = "Failed to get serial port";

struct Reader<'a, B: UsbBus> {
    container: Arc<Mutex<SerialPort<'a, B>>>,
}

fn panic_lock(e: FreeRtosError) -> ! {
    panic!("{}: {:?}", ERR_MSG, e);
}

impl<'a, B: UsbBus> rx_context for Reader<'a, B> {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize, ()> {
        fn block_thread() {
            CurrentTask::delay(Duration::ms(1));
        }

        loop {
            match self.container.lock(Duration::infinite()) {
                Ok(mut serial) => match serial.borrow_mut().read(buff) {
                    Ok(count) => {
                        if count > 0 {
                            return Ok(count);
                        } else {
                            block_thread()
                        }
                    }
                    Err(UsbError::WouldBlock) => block_thread(),
                    Err(_) => return Err(()),
                },
                Err(e) => panic_lock(e),
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
                Ok(mut serial) => {
                    let mut write_offset = 0;
                    while write_offset < src.len() {
                        match serial.write(&src[write_offset..]) {
                            Ok(len) if len > 0 => {
                                write_offset += len;
                            }
                            _ => CurrentTask::delay(Duration::ms(1)),
                        }
                    }
                    return Ok(src.len());
                }
                Err(e) => panic_lock(e),
            }
        }
    }
}

fn print_error(e: Error) {
    defmt::error!("Protobuf decode error: {}", defmt::Display2Format(&e));
}

fn flush_input<B: UsbBus>(serial_container: &mut Arc<Mutex<SerialPort<B>>>) {
    match serial_container.lock(Duration::infinite()) {
        Ok(mut s) => {
            let mut buf = [0_u8; 8];
            loop {
                if let Err(UsbError::WouldBlock) = s.read(&mut buf) {
                    break;
                }
            }
        }
        Err(e) => panic_lock(e),
    }
}

fn decode_magick<B: UsbBus>(is: &mut IStream<Reader<B>>) -> Result<(), Error> {
    match is.decode_variant() {
        Ok(v) => {
            if v != crate::protobuf::_ru_sktbelpa_pressure_self_writer_INFO_ru_sktbelpa_pressure_self_writer_INFO_MAGICK as u64 {
                Err(Error::from_str("Invalid message magick!\0"))
            } else {
                Ok(())
            }
        },
        Err(e) => Err(e)
    }
}

fn decode_msg_size<B: UsbBus>(is: &mut IStream<Reader<B>>) -> Result<usize, Error> {
    match is.decode_variant() {
        Ok(v) => {
            if v == 0 || v > 1500 {
                Err(Error::from_str("Invalid message length\0"))
            } else {
                Ok(v as usize)
            }
        }
        Err(e) => Err(e),
    }
}

pub fn protobuf_server<B: usb_device::bus::UsbBus>(
    mut serial_container: Arc<Mutex<SerialPort<B>>>,
) -> ! {
    loop {
        let msg_size = {
            let mut is = IStream::from_callback(
                Reader {
                    container: serial_container.clone(),
                },
                None,
            );

            match decode_magick(&mut is) {
                Ok(_) => {}
                Err(e) => {
                    print_error(e);
                    flush_input(&mut serial_container);
                    continue;
                }
            }

            match decode_msg_size(&mut is) {
                Ok(s) => s,
                Err(e) => {
                    print_error(e);
                    flush_input(&mut serial_container);
                    continue;
                }
            }
        };

        let message = {
            let mut is = IStream::from_callback(
                Reader {
                    container: serial_container.clone(),
                },
                Some(msg_size),
            );

            let message = match is.decode::<protobuf::ru_sktbelpa_pressure_self_writer_Request>(
                protobuf::ru_sktbelpa_pressure_self_writer_Request::fields(),
            ) {
                Ok(msg) => msg,
                Err(e) => {
                    print_error(e);
                    flush_input(&mut serial_container);
                    continue;
                }
            };

            message
        };

        {
            let mut os = OStream::from_callback(
                Writer {
                    container: serial_container.clone(),
                },
                None,
            );

            if let Err(e) = os.encode::<protobuf::ru_sktbelpa_pressure_self_writer_Request>(
                protobuf::ru_sktbelpa_pressure_self_writer_Request::fields(),
                &message,
            ) {
                print_error(e)
            }
        }
    }
}
