use core::fmt::Write;

use usb_device::UsbError;

use super::gcode;

pub enum SerialErrResult {
    OutOfMemory,
    NoData,
    Incomplead,
}

pub fn serial_process<'a, B: usb_device::bus::UsbBus, const N: usize, const M: usize>(
    serial: &mut usbd_serial::SerialPort<'static, B>,
    buf: &'a mut heapless::String<N>,
    gcode_queue: &mut heapless::Deque<gcode::GCode, M>,
    request_queue: &mut heapless::Deque<gcode::Request, M>,
) -> Result<usize, SerialErrResult> {
    use gcode::{ParceError, ParceResult};

    let mut consumed_data_len = 0;

    loop {
        match readline(serial, buf) {
            Ok(mut s) => {
                'partial: loop {
                    match super::GCode::from_string::<N>(s) {
                        Ok(ParceResult::GCode(gcode)) => {
                            gcode_queue
                                .push_back(gcode)
                                .map_err(|_| SerialErrResult::Incomplead)?;
                            break 'partial;
                        }
                        Ok(ParceResult::Request(req)) => {
                            request_queue
                                .push_back(req)
                                .map_err(|_| SerialErrResult::Incomplead)?;
                            break 'partial;
                        }
                        Ok(ParceResult::Partial(gcode, offset)) => {
                            gcode_queue
                                .push_back(gcode)
                                .map_err(|_| SerialErrResult::Incomplead)?;
                            s = &s[offset..];
                            continue 'partial;
                        }
                        Err(ParceError::Empty) => {
                            // нужно посылать "ok" даже на строки не содержащие кода
                            serial.write("ok\n\r".as_bytes()).unwrap();
                            break;
                        }
                        Err(ParceError::Error(e)) => {
                            let mut str = crate::config::HlString::new();
                            let _ = write!(&mut str, "Error: {}\n\r", e);
                            serial.write(e.as_bytes()).unwrap();
                            break 'partial;
                        }
                    }
                }

                consumed_data_len += s.len();
            }
            Err(SerialErrResult::OutOfMemory) => return Err(SerialErrResult::OutOfMemory),
            Err(SerialErrResult::NoData) | Err(SerialErrResult::Incomplead) => break,
        }
    }

    Ok(consumed_data_len)
}

fn readline<'a, B: usb_device::bus::UsbBus, const N: usize>(
    serial: &mut usbd_serial::SerialPort<'static, B>,
    buf: &'a mut heapless::String<N>,
) -> Result<&'a str, SerialErrResult> {
    static ENDLINES: [char; 3] = ['\n', '\r', '?'];

    let mut ch = [0u8; 1];
    loop {
        match serial.read(&mut ch) {
            Ok(count) => {
                if count == 1 {
                    let ch = ch[0] as char;
                    if ENDLINES.contains(&ch) {
                        // endline
                        return if ch == '?' { Ok("?") } else { Ok(&buf[..]) };
                    } else {
                        if buf.push(ch).is_ok() {
                            return Err(SerialErrResult::Incomplead);
                        } else {
                            return Err(SerialErrResult::OutOfMemory);
                        }
                    }
                } else {
                    return Err(SerialErrResult::NoData);
                }
            }
            Err(UsbError::WouldBlock) => return Err(SerialErrResult::NoData),
            Err(_) => panic!(),
        }
    }
}
