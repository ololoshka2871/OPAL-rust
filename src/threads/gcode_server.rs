use alloc::vec;
use alloc::{sync::Arc, vec::Vec};

use freertos_rust::{CurrentTask, Duration, FreeRtosError, Mutex};

use usb_device::UsbError;
use usbd_serial::SerialPort;

use super::stream::Stream;

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
        unsafe {
            let _ = freertos_rust::Task::current()
                .unwrap_unchecked()
                .take_notification(true, Duration::infinite());
        }
    }
}

pub fn gcode_server<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
    galvo_ctrl: crate::control::xy2_100::xy2_100,
) -> ! {
    let mut buf = [0u8; 1];

    let mut serial_stream = SerialStream::new(serial_container.clone(), None);

    loop {
        match serial_stream.read(&mut buf) {
            Ok(_) => {
                galvo_ctrl.set_pos(buf[0] as u16, (buf[0] ^ 0xff) as u16);
                write_responce(serial_container.clone(), &buf);
            }
            Err(e) => defmt::trace!("Serial: failed to read: {}", defmt::Debug2Format(&e)),
        }
    }
}

fn write_responce<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
    mut buf: &[u8],
) {
    loop {
        match serial_container.lock(Duration::infinite()) {
            Ok(mut serial) => match serial.write(buf) {
                Ok(len) if len > 0 => {
                    //defmt::trace!("Serial: {} bytes writen", len);
                    if len == buf.len() {
                        return;
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
