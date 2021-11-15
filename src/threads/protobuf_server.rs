use core::borrow::BorrowMut;

use alloc::{boxed::Box, sync::Arc, vec};

use freertos_rust::{CurrentTask, Duration, FreeRtosError, FreeRtosUtils, Mutex};

use nanopb_rs::{dyn_fields::TxRepeated, pb_decode::rx_context, Error, IStream, OStream};

use usb_device::{class_prelude::UsbBus, UsbError};

use usbd_serial::SerialPort;

use crate::{
    protobuf::{self, *},
    settings::settings_action,
};

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

fn print_error(e: Error) {
    defmt::error!("Protobuf decode error: {}", defmt::Display2Format(&e));
}

fn decode_magick<B: UsbBus>(is: &mut IStream<Reader<B>>) -> Result<(), Error> {
    match is.stream().decode_variant() {
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
    match is.stream().decode_variant() {
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
    serial_container: Arc<Mutex<SerialPort<B>>>,
) -> ! {
    loop {
        let msg_size = match recive_md_header(serial_container.clone()) {
            Ok(size) => size,
            Err(e) => {
                print_error(e);
                continue;
            }
        };

        let request = match recive_message_body(serial_container.clone(), msg_size) {
            Ok(request) => request,
            Err(_) => continue,
        };

        defmt::info!("Nanopb: Request:\n{}", defmt::Debug2Format(&request));

        let response = {
            let id = request.id;
            if let Ok(r) = process_requiest(request, new_response(id)) {
                r
            } else {
                // FIXME
                defmt::error!("Failed to build responce");
                continue;
            }
        };

        defmt::info!("Nanopb: Response:\n{}", defmt::Debug2Format(&response));

        if let Err(e) = write_responce(serial_container.clone(), response) {
            print_error(e);
        }
    }
}

fn write_responce<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
    response: ru_sktbelpa_pressure_self_writer_Response,
) -> Result<(), Error> {
    let size = ru_sktbelpa_pressure_self_writer_Response::get_size(&response);
    let mut buf = vec![0_u8; size + 1 + core::mem::size_of::<u64>()];
    let buf = buf.as_mut_slice();
    let mut os = OStream::from_buffer(buf);

    if let Err(_) = os
        .stream()
        .write(&[ru_sktbelpa_pressure_self_writer_INFO_MAGICK])
    {
        return Err(Error::from_str("Failed to write magick\0"));
    }

    if let Err(_) = os.stream().encode_varint(size as u64) {
        return Err(Error::from_str("Failed to encode size\0"));
    }

    if let Err(e) = os
        .stream()
        .encode::<ru_sktbelpa_pressure_self_writer_Response>(
            ru_sktbelpa_pressure_self_writer_Response::fields(),
            &response,
        )
    {
        return Err(e);
    }

    let mut buf = &buf[..os.stram_size()];
    loop {
        match serial_container.lock(Duration::infinite()) {
            Ok(mut serial) => match serial.write(buf) {
                Ok(len) if len > 0 => {
                    defmt::trace!("Serial: {} bytes writen", len);
                    if len == buf.len() {
                        return Ok(());
                    }
                    buf = &buf[len..];
                }
                _ => {}
            },
            Err(e) => panic_lock(e),
        }
        CurrentTask::delay(Duration::ms(1));
    }
}

fn recive_message_body<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
    msg_size: usize,
) -> Result<ru_sktbelpa_pressure_self_writer_Request, ()> {
    let mut is = IStream::from_callback(
        Reader {
            container: serial_container,
        },
        Some(msg_size),
    );

    match is
        .stream()
        .decode::<ru_sktbelpa_pressure_self_writer_Request>(
            ru_sktbelpa_pressure_self_writer_Request::fields(),
        ) {
        Ok(msg) => {
            if !(msg.deviceID == ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID
                || msg.deviceID == ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER)
            {
                defmt::error!("Protobuf: unknown target device id: 0x{:X}", msg.deviceID);
                return Err(());
            }

            if msg.deviceID != ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER
                && msg.protocolVersion != ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION
            {
                defmt::error!(
                    "Protobuf: unsupported protocol version {}",
                    msg.protocolVersion
                );
                return Err(());
            }
            Ok(msg)
        }
        Err(e) => {
            print_error(e);
            is.stream().flush();
            Err(())
        }
    }
}

fn recive_md_header<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
) -> Result<usize, Error> {
    let mut is = IStream::from_callback(
        Reader {
            container: serial_container,
        },
        None,
    );

    match decode_magick(&mut is) {
        Ok(_) => {}
        Err(e) => {
            is.stream().flush();
            return Err(e);
        }
    }

    match decode_msg_size(&mut is) {
        Ok(s) => Ok(s),
        Err(e) => {
            is.stream().flush();
            Err(e)
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
) -> Result<ru_sktbelpa_pressure_self_writer_Response, ()> {
    if req.has_writeSettings {
        resp.has_getSettings = true;

        fill_settings(&mut resp.getSettings)?;
    }

    Ok(resp)
}

fn fill_settings(
    settings_resp: &mut ru_sktbelpa_pressure_self_writer_SettingsResponse,
) -> Result<(), ()> {
    struct FloatSender {
        container: Box<[f32]>,
        iterator: usize,
        fields: &'static pb_msgdesc_t,
    }

    impl FloatSender {
        fn new(container: Box<[f32]>, fields: &'static pb_msgdesc_t) -> Self {
            Self {
                container,
                iterator: 0,
                fields,
            }
        }
    }

    impl TxRepeated for FloatSender {
        fn reset(&mut self) {
            self.iterator = 0;
        }

        fn has_next(&self) -> bool {
            self.iterator < self.container.len()
        }

        fn encode_next(
            &mut self,
            out_stream: &mut nanopb_rs::pb_encode::pb_ostream_t,
        ) -> Result<(), Error> {
            let data = self.container[self.iterator];
            self.iterator += 1;
            out_stream.encode_f32(data)
        }

        fn fields(&self) -> &'static pb_msgdesc_t {
            self.fields
        }
    }

    settings_action(Duration::ms(5), |settings| {
        settings_resp.Serial = settings.serial;

        settings_resp.PMesureTime_ms = settings.pmesure_time_ms;
        settings_resp.TMesureTime_ms = settings.tmesure_time_ms;

        settings_resp.Fref = settings.fref;

        settings_resp.PEnabled = settings.p_enabled;
        settings_resp.TEnabled = settings.t_enabled;

        settings_resp.PCoefficients = ru_sktbelpa_pressure_self_writer_PCoefficientsGet {
            Fp0: settings.pcoefficients.Fp0,
            Ft0: settings.pcoefficients.Ft0,
            A: nanopb_rs::dyn_fields::new_tx_repeated_callback(Box::new(FloatSender::new(
                Box::from(settings.pcoefficients.A),
                protobuf::ru_sktbelpa_pressure_self_writer_PCoefficientsGet::fields(),
            ))),
        };

        settings_resp.TCoefficients = ru_sktbelpa_pressure_self_writer_T5CoefficientsGet {
            T0: settings.tcoefficients.T0,
            F0: settings.tcoefficients.F0,
            C: nanopb_rs::dyn_fields::new_tx_repeated_callback(Box::new(FloatSender::new(
                Box::from(settings.tcoefficients.C),
                protobuf::ru_sktbelpa_pressure_self_writer_T5CoefficientsGet::fields(),
            ))),
        };
    })
    .map_err(|_| ())
}
