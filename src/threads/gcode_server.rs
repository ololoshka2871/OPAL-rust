use alloc::vec;
use alloc::{format, string::String, sync::Arc, vec::Vec};

use freertos_rust::{CurrentTask, Duration, FreeRtosError, Mutex, Queue};

use usb_device::UsbError;
use usbd_serial::SerialPort;

use crate::gcode::{self, GCode, ParceError, ParceResult, Request};

use super::stream::Stream;

struct SerialStream<'a, B: usb_device::bus::UsbBus> {
    serial_container: Arc<Mutex<&'a mut SerialPort<'a, B>>>,
    max_size: Option<usize>,
    endlines: Vec<char>,
}

impl<'a, B: usb_device::bus::UsbBus> Stream<FreeRtosError> for SerialStream<'a, B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<(), FreeRtosError> {
        loop {
            match self.serial_container.lock(Duration::infinite()) {
                Ok(mut serial) => match serial.read(buf) {
                    Ok(count) => {
                        if count > 0 {
                            defmt::trace!("Serial: {} bytes ressived", count);
                            return Ok(());
                        } else {
                            Self::block_thread()
                        }
                    }
                    Err(UsbError::WouldBlock) => Self::block_thread(),
                    Err(_) => panic!(),
                },
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

    fn read_line(&mut self, max_len: Option<usize>) -> Result<String, FreeRtosError> {
        let mut resut = String::new();
        loop {
            CurrentTask::delay(Duration::ms(1));
            match self.serial_container.lock(Duration::zero()) {
                Ok(mut serial) => {
                    let mut buf = [0u8; 1];
                    match serial.read(&mut buf) {
                        Ok(count) => {
                            if count > 0 {
                                let ch = buf[0] as char;
                                resut.push(ch);
                                if self.endlines.contains(&ch) {
                                    if resut.is_empty() {
                                        continue; // empty string
                                    } else {
                                        return Ok(resut);
                                    }
                                } else {
                                    if let Some(ml) = max_len {
                                        if resut.len() >= ml {
                                            return Err(FreeRtosError::OutOfMemory);
                                        }
                                    }
                                }
                            } else {
                                Self::block_thread()
                            }
                        }
                        Err(UsbError::WouldBlock) => Self::block_thread(),
                        Err(_) => panic!(),
                    }
                }
                Err(FreeRtosError::MutexTimeout) => { /* ok */ }
                Err(e) => return Err(e),
            }
        }
    }
}

impl<'a, B: usb_device::bus::UsbBus> SerialStream<'a, B> {
    fn new(
        serial_container: Arc<Mutex<&'a mut SerialPort<'a, B>>>,
        max_size: Option<usize>,
        endlines: Vec<char>,
    ) -> Self {
        Self {
            serial_container,
            max_size,
            endlines,
        }
    }

    fn block_thread() {
        unsafe {
            let _ = freertos_rust::Task::current()
                .unwrap_unchecked()
                .take_notification(true, Duration::infinite());
        }
    }
}

pub fn gcode_server<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<&'static mut SerialPort<B>>>,
    gcode_tx_queue: Arc<Queue<GCode>>,
    req_tx_queue: Arc<Queue<Request>>,
) -> ! {
    let mut serial_stream =
        SerialStream::new(serial_container.clone(), None, vec!['\n', '\r', '?']);

    loop {
        match serial_stream.read_line(Some(gcode::MAX_LEN)) {
            Ok(s) => match GCode::from_string(s.as_str()) {
                Ok(ParceResult::GCode(gcode)) => {
                    let _ = gcode_tx_queue.send(gcode, Duration::infinite());
                }
                Ok(ParceResult::Request(req)) => {
                    let _ = req_tx_queue.send(req, Duration::zero());
                }
                Err(ParceError::Empty) => {
                    // нужно посылать "ok" даже на строки не содержащие кода
                    defmt::trace!(
                        "Empty command: {}",
                        crate::support::defmt_string::DefmtString(&s)
                    );
                    write_responce(&serial_container, "ok\n");
                }
                Err(ParceError::Error(e)) => {
                    write_responce(&serial_container, format!("Error: {:?}\n", e).as_str())
                }
            },
            Err(e) => write_responce(&serial_container, format!("Error: {:?}\n\r", e).as_str()),
        }
    }
}

pub fn write_responce<B: usb_device::bus::UsbBus>(
    serial_container: &Arc<Mutex<&'static mut SerialPort<B>>>,
    mut text: &str,
) {
    loop {
        match serial_container.lock(Duration::zero()) {
            Ok(mut serial) => match serial.write(text.as_bytes()) {
                Ok(len) if len > 0 => {
                    defmt::trace!("Serial: {} bytes writen", len);
                    if len == text.len() {
                        return;
                    }
                    text = &text[len..];
                }
                _ => {}
            },
            Err(FreeRtosError::MutexTimeout) => CurrentTask::delay(Duration::ms(1)),
            Err(e) => panic!("{:?}", e),
        }
        CurrentTask::delay(Duration::ms(1));
    }
}
