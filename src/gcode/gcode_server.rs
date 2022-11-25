use core::fmt::Write;

use usb_device::UsbError;

use super::gcode;

pub enum SerialErrResult {
    OutOfMemory,
    NoData,
    Incomplead(usize),
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

    match readline(serial, buf) {
        Ok(mut s) => 'partial: loop {
            match super::GCode::from_string::<N>(s) {
                Ok(ParceResult::GCode(gcode)) => {
                    gcode_pusher(gcode)
                        .map_err(|_| SerialErrResult::Incomplead(consumed_data_len))?;
                    consumed_data_len += s.len();
                    break 'partial;
                }
                Ok(ParceResult::Request(req)) => {
                    request_pusher(req)
                        .map_err(|_| SerialErrResult::Incomplead(consumed_data_len))?;
                    consumed_data_len += s.len();
                    break 'partial;
                }
                Ok(ParceResult::Partial(gcode, offset)) => {
                    gcode_pusher(gcode)
                        .map_err(|_| SerialErrResult::Incomplead(consumed_data_len))?;
                    consumed_data_len += offset;
                    s = &s[offset..];
                    continue 'partial;
                }
                Err(ParceError::Empty) => {
                    consumed_data_len += s.len();
                    break 'partial;
                }
                Err(ParceError::Error(e)) => {
                    consumed_data_len += s.len();
                    let mut str = crate::config::HlString::new();
                    let _ = write!(&mut str, "Error: {}\n\r", e);
                    serial.write(e.as_bytes()).unwrap();
                    break 'partial;
                }
            }
        },
        Err(SerialErrResult::OutOfMemory) => return Err(SerialErrResult::OutOfMemory),
        Err(SerialErrResult::NoData) => {}
        Err(SerialErrResult::Incomplead(_)) => unreachable!(),
    }

    Ok(consumed_data_len)
}

fn readline<'a, B: usb_device::bus::UsbBus, const N: usize>(
    serial: &mut usbd_serial::SerialPort<'static, B>,
    buf: &'a mut heapless::String<N>,
) -> Result<&'a str, SerialErrResult> {
    static ENDLINES: [char; 3] = ['\n', '\r', '?'];

    // если строка уже кончается на что-то из ENDLINES
    if let Some(ch) = buf.chars().last() {
        if ENDLINES.contains(&ch) {
            return Ok(&buf[..]);
        }
    }

    let mut ch = [0u8; 1];
    loop {
        match serial.read(&mut ch) {
            Ok(count) => {
                if count == 1 {
                    let ch = ch[0] as char;
                    if ch == '?' {
                        return Ok("?");
                    }
                    if buf.push(ch).is_err() {
                        return Err(SerialErrResult::OutOfMemory);
                    }
                    if ENDLINES.contains(&ch) {
                        // endline
                        return Ok(&buf[..]);
                    } else {
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
