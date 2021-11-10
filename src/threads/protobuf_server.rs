use core::borrow::BorrowMut;

use alloc::boxed::Box;
use alloc::sync::Arc;

use freertos_rust::{CurrentTask, Duration, FreeRtosError, FreeRtosUtils, Mutex};

use nanopb_rs::dyn_fields::TxRepeated;
use nanopb_rs::{pb_decode::rx_context, pb_encode::tx_context, Error, IStream, OStream};

use usb_device::{class_prelude::UsbBus, UsbError};

use usbd_serial::SerialPort;

use crate::protobuf::*;

use crate::protobuf::Sizable;

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
                Ok(mut serial) => {
                    let ser = serial.borrow_mut();
                    match ser.read(buff) {
                        Ok(count) => {
                            if count > 0 {
                                defmt::trace!("Serial: {} bytes ressived", count);
                                return Ok(count);
                            } else {
                                block_thread()
                            }
                        }
                        Err(UsbError::WouldBlock) => block_thread(),
                        Err(_) => return Err(()),
                    }
                }
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
                                defmt::trace!("Serial: {} bytes writen", len);
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
                // s.flush() - это не то
                match s.read(&mut buf) {
                    Ok(len) => {
                        if len < buf.len() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            defmt::trace!("Serial: input buffer flushed");
        }
        Err(e) => panic_lock(e),
    }
}

fn decode_magick<B: UsbBus>(is: &mut IStream<Reader<B>>) -> Result<(), Error> {
    match is.decode_variant() {
        Ok(v) => {
            if v != crate::protobuf::ru_sktbelpa_pressure_self_writer_INFO_MAGICK as u64 {
                Err(Error::from_str("Invalid message magick!\0"))
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e),
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

        let request = {
            let mut is = IStream::from_callback(
                Reader {
                    container: serial_container.clone(),
                },
                Some(msg_size),
            );

            let message = match is.decode::<ru_sktbelpa_pressure_self_writer_Request>(
                ru_sktbelpa_pressure_self_writer_Request::fields(),
            ) {
                Ok(msg) => {
                    if !(msg.deviceID
                        == ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID
                        || msg.deviceID == ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER)
                    {
                        defmt::error!("Protobuf: unknown target device id: 0x{:X}", msg.deviceID);
                        continue;
                    }
                    if msg.deviceID != ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER
                        && msg.protocolVersion
                            != ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION
                    {
                        defmt::error!(
                            "Protobuf: unsupported protocol version {}",
                            msg.protocolVersion
                        );
                        continue;
                    }
                    msg
                }
                Err(e) => {
                    print_error(e);
                    flush_input(&mut serial_container);
                    continue;
                }
            };

            message
        };

        defmt::info!("Nanopb: got request: {}", defmt::Debug2Format(&request));

        {
            let response = {
                let id = request.id;
                process_requiest(request, new_response(id))
            };

            let mut os = OStream::from_callback(
                Writer {
                    container: serial_container.clone(),
                },
                None,
            );

            if let Err(_) = os.write(&[ru_sktbelpa_pressure_self_writer_INFO_MAGICK]) {
                print_error(nanopb_rs::Error::from_str("Failed to write magick\0"));
                continue;
            }

            if let Err(_) = os.encode_varint(ru_sktbelpa_pressure_self_writer_Response::get_size(
                &response,
            ) as u64)
            {
                print_error(nanopb_rs::Error::from_str("Failed to encode size\0"));
                continue;
            }

            defmt::info!(
                "Nanopb: writing response: {}",
                defmt::Debug2Format(&response)
            );
            if let Err(e) = os.encode::<ru_sktbelpa_pressure_self_writer_Response>(
                ru_sktbelpa_pressure_self_writer_Response::fields(),
                &response,
            ) {
                print_error(e)
            }
        }
    }
}

fn new_response(id: u32) -> ru_sktbelpa_pressure_self_writer_Response {
    let mut res = ru_sktbelpa_pressure_self_writer_Response::default();

    res.id = id;
    res.deviceID = ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID;
    res.protocolVersion = ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION;
    res.Global_status = ru_sktbelpa_pressure_self_writer_STATUS_OK;
    res.timestamp = FreeRtosUtils::get_tick_count() as u64;

    res
}

fn process_requiest(
    req: ru_sktbelpa_pressure_self_writer_Request,
    mut resp: ru_sktbelpa_pressure_self_writer_Response,
) -> ru_sktbelpa_pressure_self_writer_Response {
    if req.has_writeSettings {
        resp.has_getSettings = true;

        fill_settings(&mut resp.getSettings);
    }

    resp
}

fn fill_settings(settings_resp: &mut ru_sktbelpa_pressure_self_writer_SettingsResponse) {
    struct FloatIterator(u32);

    impl TxRepeated<f32> for FloatIterator {
        fn next_item(&mut self) -> Option<f32> {
            if self.0 != 0 {
                self.0 -= 1;
                Some(0.0)
            } else {
                None
            }
        }

        fn encode_body(
            &self,
            out_stream: &mut nanopb_rs::pb_encode::pb_ostream_t,
            data: f32,
        ) -> bool {
            unsafe {
                let t = *(&data as *const f32 as *const u32) as u64;
                nanopb_rs::pb_encode::pb_encode_varint(out_stream, t)
            }
        }

        fn fields(&self) -> &'static pb_msgdesc_t {
            ru_sktbelpa_pressure_self_writer_PCoefficientsGet::fields()
        }
    }

    settings_resp.Serial = 1;

    settings_resp.PMesureTime_ms = 1000;
    settings_resp.TMesureTime_ms = 1000;

    settings_resp.Fref = 16000000;

    settings_resp.PEnabled = true;
    settings_resp.TEnabled = true;

    settings_resp.PCoefficients = ru_sktbelpa_pressure_self_writer_PCoefficientsGet {
        Fp0: 0.0,
        Ft0: 0.0,
        A: nanopb_rs::dyn_fields::new_tx_repeated_callback(Box::new(FloatIterator(16))),
    };

    settings_resp.TCoefficients = ru_sktbelpa_pressure_self_writer_T5CoefficientsGet {
        T0: 0.0,
        F0: 0.0,
        C: nanopb_rs::dyn_fields::new_tx_repeated_callback(Box::new(FloatIterator(6))),
    };
}
