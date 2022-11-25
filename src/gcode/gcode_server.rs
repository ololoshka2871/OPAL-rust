use core::fmt::Write;

use usb_device::UsbError;

use super::gcode;

pub enum SerialErrResult {
    OutOfMemory,
    NoData,
    Incomplead,
}

pub fn serial_process<'a, B, GP, RP, const N: usize>(
    serial: &mut usbd_serial::SerialPort<'static, B>,
    buf: &'a mut heapless::String<N>,
    mut gcode_pusher: GP,
    mut request_pusher: RP,
) -> Result<usize, SerialErrResult>
where
    GP: FnMut(gcode::GCode) -> Result<(), gcode::GCode>,
    RP: FnMut(gcode::Request) -> Result<(), gcode::Request>,
    B: usb_device::bus::UsbBus,
{
    use gcode::{ParceError, ParceResult};

    let mut consumed_data_len = 0;

    'outer: loop {
        match readline(serial, buf) {
            Ok(mut s) => {
                'partial: loop {
                    match super::GCode::from_string::<N>(s) {
                        Ok(ParceResult::GCode(gcode)) => {
                            consumed_data_len += s.len();
                            gcode_pusher(gcode).map_err(|_| SerialErrResult::Incomplead)?;
                            break 'partial;
                        }
                        Ok(ParceResult::Request(req)) => {
                            consumed_data_len += s.len();
                            request_pusher(req).map_err(|_| SerialErrResult::Incomplead)?;
                            break 'partial;
                        }
                        Ok(ParceResult::Partial(gcode, offset)) => {
                            consumed_data_len += offset;
                            gcode_pusher(gcode).map_err(|_| SerialErrResult::Incomplead)?;
                            s = &s[offset..];
                            continue 'partial;
                        }
                        Err(ParceError::Empty) => {
                            // нужно посылать "ok" даже на строки не содержащие кода
                            //serial.write("ok\n\r".as_bytes()).unwrap();
                            consumed_data_len += s.len();
                            break 'outer;
                        }
                        Err(ParceError::Error(e)) => {
                            consumed_data_len += s.len();
                            let mut str = crate::config::HlString::new();
                            let _ = write!(&mut str, "Error: {}\n\r", e);
                            serial.write(e.as_bytes()).unwrap();
                            break 'partial;
                        }
                    }
                }
            }
            Err(SerialErrResult::OutOfMemory) => return Err(SerialErrResult::OutOfMemory),
            Err(SerialErrResult::NoData) => break 'outer,
            Err(SerialErrResult::Incomplead) => {}
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
                        if buf.push(ch).is_err() {
                            return Err(SerialErrResult::OutOfMemory);
                        }
                        // continue read string
                    }
                } else {
                    return Err(SerialErrResult::NoData);
                }
            }
            Err(UsbError::WouldBlock) => {
                return Err(SerialErrResult::NoData);
            }
            Err(_) => panic!(),
        }
    }
}
