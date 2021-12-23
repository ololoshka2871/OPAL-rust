use core::fmt::Display;

use alloc::vec;
use alloc::{sync::Arc, vec::Vec};

use freertos_rust::{CurrentTask, Duration, FreeRtosError, Mutex};

use prost::EncodeError;
use usb_device::UsbError;
use usbd_serial::SerialPort;

use crate::{
    protobuf::{self, Stream},
    workmodes::output_storage::OutputStorage,
};

struct SerialStream<'a, B: usb_device::bus::UsbBus> {
    serial_container: Arc<Mutex<SerialPort<'a, B>>>,
    max_size: Option<usize>,
}

impl<'a, B: usb_device::bus::UsbBus> Stream<FreeRtosError> for SerialStream<'a, B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<(), FreeRtosError> {
        loop {
            match self.serial_container.lock(Duration::infinite()) {
                Ok(mut serial) => {
                    match serial.read(buf) {
                        Ok(count) => {
                            if count > 0 {
                                //defmt::trace!("Serial: {} bytes ressived", count);
                                return Ok(());
                            } else {
                                Self::block_thread()
                            }
                        }
                        Err(UsbError::WouldBlock) => Self::block_thread(),
                        Err(_) => panic!(),
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn read_all(&mut self) -> Result<Vec<u8>, FreeRtosError> {
        if let Some(max_size) = &self.max_size {
            let mut data = vec![0u8; *max_size];
            match self.read(data.as_mut_slice()) {
                Ok(_) => Ok(data),
                Err(e) => Err(e),
            }
        } else {
            Err(FreeRtosError::OutOfMemory)
        }
    }
}

impl<'a, B: usb_device::bus::UsbBus> SerialStream<'a, B> {
    fn new(serial_container: Arc<Mutex<SerialPort<'a, B>>>, max_size: Option<usize>) -> Self {
        Self {
            serial_container,
            max_size,
        }
    }

    fn block_thread() {
        let _ = freertos_rust::Task::current()
            .unwrap()
            .take_notification(true, Duration::infinite());
    }
}

pub fn protobuf_server<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
    output: Arc<Mutex<OutputStorage>>,
) -> ! {
    loop {
        let msg_size = match protobuf::recive_md_header(&mut SerialStream::new(
            serial_container.clone(),
            None,
        )) {
            Ok(size) => size,
            Err(e) => {
                print_error(e);
                continue;
            }
        };

        let request = match protobuf::recive_message_body(&mut SerialStream::new(
            serial_container.clone(),
            Some(msg_size),
        )) {
            Ok(request) => request,
            Err(e) => {
                print_error(e);
                continue;
            }
        };

        let id = request.id;

        //defmt::info!("Protobuf: Request:\n{}", defmt::Debug2Format(&request));

        let response = {
            let id = request.id;
            match protobuf::process_requiest(request, protobuf::new_response(id), &output) {
                Ok(r) => r,
                Err(_) => {
                    defmt::error!("Failed to generate response");
                    continue;
                }
            }
        };

        //defmt::info!("Protobuf: Response:\n{}", defmt::Debug2Format(&response));

        if let Err(e) = write_responce(serial_container.clone(), response) {
            print_error(e);
        }

        defmt::trace!("Protobuf: message id = {} processed succesfully", id);
    }
}

fn print_error<E: Display>(e: E) {
    defmt::error!("Protobuf error: {}", defmt::Display2Format(&e));
}

fn write_responce<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
    response: protobuf::Response,
) -> Result<(), EncodeError> {
    let data = protobuf::encode_md_message(response)?;
    let mut buf = data.as_slice();

    loop {
        match serial_container.lock(Duration::infinite()) {
            Ok(mut serial) => match serial.write(buf) {
                Ok(len) if len > 0 => {
                    //defmt::trace!("Serial: {} bytes writen", len);
                    if len == buf.len() {
                        return Ok(());
                    }
                    buf = &buf[len..];
                }
                _ => {}
            },
            Err(e) => panic!("{:?}", e),
        }
        CurrentTask::delay(Duration::ms(1));
    }
}
