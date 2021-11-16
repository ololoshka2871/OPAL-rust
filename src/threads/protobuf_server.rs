use core::{borrow::BorrowMut, mem};

use alloc::{format, string::String, sync::Arc, vec};

use freertos_rust::{CurrentTask, Duration, FreeRtosError, FreeRtosUtils, Mutex};

use nanopb_rs::{pb_decode::rx_context, Error, IStream, OStream};

use usb_device::{class_prelude::UsbBus, UsbError};

use usbd_serial::SerialPort;

use crate::protobuf::*;

static ERR_MSG: &str = "Failed to get serial port";

static MAX_MT: u32 = 5000;
static MIN_MT: u32 = 10;

static F_REF_BASE: u32 = 16000000;
static F_REF_DELTA: u32 = 500;

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

        //defmt::info!("Nanopb: Request:\n{}", defmt::Debug2Format(&request));
        defmt::info!("Nanopb: Request id={}", request.id);

        let response = {
            let id = request.id;
            match process_requiest(request, new_response(id)) {
                Ok(r) => r,
                Err(_) => {
                    defmt::error!("Failed to generate response");
                    continue;
                }
            }
        };

        //defmt::info!("Nanopb: Response:\n{}", defmt::Debug2Format(&response));
        defmt::info!("Nanopb: Response ready");

        if let Err(e) = write_responce(serial_container.clone(), response) {
            print_error(e);
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

    let mut res: ru_sktbelpa_pressure_self_writer_Request =
        unsafe { mem::MaybeUninit::zeroed().assume_init() };
    match is
        .stream()
        .decode(&mut res, ru_sktbelpa_pressure_self_writer_Request::fields())
    {
        Ok(_) => Ok(res),
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
    if !(req.deviceID == ru_sktbelpa_pressure_self_writer_INFO_PRESSURE_SELF_WRITER_ID
        || req.deviceID == ru_sktbelpa_pressure_self_writer_INFO_ID_DISCOVER)
    {
        defmt::error!("Protobuf: unknown target device id: 0x{:X}", req.deviceID);

        resp.Global_status =
            crate::protobuf::ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR;
        return Ok(resp);
    }

    if req.protocolVersion != ru_sktbelpa_pressure_self_writer_INFO_PROTOCOL_VERSION {
        defmt::warn!(
            "Protobuf: unsupported protocol version {}",
            req.protocolVersion
        );
        resp.Global_status =
            crate::protobuf::ru_sktbelpa_pressure_self_writer_STATUS_PROTOCOL_ERROR;

        return Ok(resp);
    }

    if req.has_writeSettings {
        resp.has_getSettings = true;
        //defmt::info!("Update settings: {}", defmt::Debug2Format(&req.writeSettings));
        if let Err(e) = update_settings(&req.writeSettings) {
            defmt::error!("Set settings error: {}", defmt::Debug2Format(&e));
            resp.Global_status =
                crate::protobuf::ru_sktbelpa_pressure_self_writer_STATUS_ERRORS_IN_SUBCOMMANDS;
        }
        fill_settings(&mut resp.getSettings)?;
        //defmt::info!("Settings responce: {}", defmt::Debug2Format(&resp.getSettings));
    }

    Ok(resp)
}

fn fill_settings(
    settings_resp: &mut ru_sktbelpa_pressure_self_writer_SettingsResponse,
) -> Result<(), ()> {
    crate::settings::settings_action(Duration::ms(1), |settings| {
        settings_resp.Serial = settings.serial;

        settings_resp.PMesureTime_ms = settings.pmesure_time_ms;
        settings_resp.TMesureTime_ms = settings.tmesure_time_ms;

        settings_resp.Fref = settings.fref;

        settings_resp.PEnabled = settings.p_enabled;
        settings_resp.TEnabled = settings.t_enabled;

        settings_resp.PCoefficients = ru_sktbelpa_pressure_self_writer_PCoefficients {
            has_Fp0: true,
            Fp0: settings.pcoefficients.Fp0,
            has_Ft0: true,
            Ft0: settings.pcoefficients.Ft0,

            has_A0: true,
            A0: settings.pcoefficients.A[0],
            has_A1: true,
            A1: settings.pcoefficients.A[1],
            has_A2: true,
            A2: settings.pcoefficients.A[2],
            has_A3: true,
            A3: settings.pcoefficients.A[3],
            has_A4: true,
            A4: settings.pcoefficients.A[4],
            has_A5: true,
            A5: settings.pcoefficients.A[5],
            has_A6: true,
            A6: settings.pcoefficients.A[6],
            has_A7: true,
            A7: settings.pcoefficients.A[7],
            has_A8: true,
            A8: settings.pcoefficients.A[8],
            has_A9: true,
            A9: settings.pcoefficients.A[9],
            has_A10: true,
            A10: settings.pcoefficients.A[10],
            has_A11: true,
            A11: settings.pcoefficients.A[11],
            has_A12: true,
            A12: settings.pcoefficients.A[12],
            has_A13: true,
            A13: settings.pcoefficients.A[13],
            has_A14: true,
            A14: settings.pcoefficients.A[14],
            has_A15: true,
            A15: settings.pcoefficients.A[15],
        };

        settings_resp.TCoefficients = ru_sktbelpa_pressure_self_writer_T5Coefficients {
            has_T0: true,
            T0: settings.tcoefficients.T0,
            has_F0: true,
            F0: settings.tcoefficients.F0,

            has_C1: true,
            C1: settings.tcoefficients.C[0],
            has_C2: true,
            C2: settings.tcoefficients.C[1],
            has_C3: true,
            C3: settings.tcoefficients.C[2],
            has_C4: true,
            C4: settings.tcoefficients.C[3],
            has_C5: true,
            C5: settings.tcoefficients.C[4],
        };
        Ok(())
    })
    .map_err(|_: crate::settings::SettingActionError<()>| ())
}

fn update_settings(
    ws: &ru_sktbelpa_pressure_self_writer_WriteSettingsReq,
) -> Result<(), crate::settings::SettingActionError<String>> {
    let mut need_write = false;

    // verfy values
    if ws.has_PMesureTime_ms && (ws.PMesureTime_ms > MAX_MT || ws.PMesureTime_ms < MIN_MT) {
        return Err(crate::settings::SettingActionError::ActionError(format!(
            "Pressure measure time {} is out of range {} - {}",
            ws.PMesureTime_ms, MIN_MT, MAX_MT
        )));
    }

    if ws.has_TMesureTime_ms && (ws.TMesureTime_ms > MAX_MT || ws.TMesureTime_ms < MIN_MT) {
        return Err(crate::settings::SettingActionError::ActionError(format!(
            "Temperature measure time {} is out of range {} - {}",
            ws.PMesureTime_ms, MIN_MT, MAX_MT
        )));
    }

    if ws.has_Fref && (ws.Fref > F_REF_BASE + F_REF_DELTA || ws.Fref < F_REF_BASE - F_REF_DELTA) {
        return Err(crate::settings::SettingActionError::ActionError(format!(
            "Reference frequency {} is too different from base {} +/- {}",
            ws.Fref, F_REF_BASE, F_REF_DELTA
        )));
    }

    crate::settings::settings_action(Duration::ms(1), |settings| {
        if ws.has_PMesureTime_ms {
            settings.pmesure_time_ms = ws.PMesureTime_ms;
            need_write = true;
        }

        if ws.has_TMesureTime_ms {
            settings.tmesure_time_ms = ws.TMesureTime_ms;
            need_write = true;
        }

        if ws.has_Fref {
            settings.fref = ws.Fref;
            need_write = true;
        }

        if ws.has_Serial {
            settings.serial = ws.Serial;
            need_write = true;
        }

        if ws.has_PEnabled {
            settings.p_enabled = ws.PEnabled;
            need_write = true;
        }

        if ws.has_TEnabled {
            settings.t_enabled = ws.TEnabled;
            need_write = true;
        }

        if ws.has_PCoefficients {
            if ws.PCoefficients.has_Fp0 {
                settings.pcoefficients.Fp0 = ws.PCoefficients.Fp0;
                need_write = true;
            }

            if ws.PCoefficients.has_Ft0 {
                settings.pcoefficients.Ft0 = ws.PCoefficients.Ft0;
                need_write = true;
            }

            if ws.PCoefficients.has_A0 {
                settings.pcoefficients.A[0] = ws.PCoefficients.A0;
                need_write = true;
            }

            if ws.PCoefficients.has_A1 {
                settings.pcoefficients.A[1] = ws.PCoefficients.A1;
                need_write = true;
            }

            if ws.PCoefficients.has_A2 {
                settings.pcoefficients.A[2] = ws.PCoefficients.A2;
                need_write = true;
            }

            if ws.PCoefficients.has_A3 {
                settings.pcoefficients.A[3] = ws.PCoefficients.A3;
                need_write = true;
            }

            if ws.PCoefficients.has_A4 {
                settings.pcoefficients.A[4] = ws.PCoefficients.A4;
                need_write = true;
            }

            if ws.PCoefficients.has_A5 {
                settings.pcoefficients.A[5] = ws.PCoefficients.A5;
                need_write = true;
            }

            if ws.PCoefficients.has_A6 {
                settings.pcoefficients.A[6] = ws.PCoefficients.A6;
                need_write = true;
            }

            if ws.PCoefficients.has_A7 {
                settings.pcoefficients.A[7] = ws.PCoefficients.A7;
                need_write = true;
            }

            if ws.PCoefficients.has_A8 {
                settings.pcoefficients.A[8] = ws.PCoefficients.A8;
                need_write = true;
            }

            if ws.PCoefficients.has_A9 {
                settings.pcoefficients.A[9] = ws.PCoefficients.A9;
                need_write = true;
            }

            if ws.PCoefficients.has_A10 {
                settings.pcoefficients.A[10] = ws.PCoefficients.A10;
                need_write = true;
            }

            if ws.PCoefficients.has_A11 {
                settings.pcoefficients.A[11] = ws.PCoefficients.A11;
                need_write = true;
            }

            if ws.PCoefficients.has_A12 {
                settings.pcoefficients.A[12] = ws.PCoefficients.A12;
                need_write = true;
            }

            if ws.PCoefficients.has_A13 {
                settings.pcoefficients.A[13] = ws.PCoefficients.A13;
                need_write = true;
            }

            if ws.PCoefficients.has_A14 {
                settings.pcoefficients.A[14] = ws.PCoefficients.A14;
                need_write = true;
            }

            if ws.PCoefficients.has_A15 {
                settings.pcoefficients.A[15] = ws.PCoefficients.A15;
                need_write = true;
            }
        }

        if ws.has_TCoefficients {
            if ws.TCoefficients.has_F0 {
                settings.tcoefficients.F0 = ws.TCoefficients.F0;
                need_write = true;
            }

            if ws.TCoefficients.has_T0 {
                settings.tcoefficients.T0 = ws.TCoefficients.T0;
                need_write = true;
            }

            if ws.TCoefficients.has_C1 {
                settings.tcoefficients.C[0] = ws.TCoefficients.C1;
                need_write = true;
            }

            if ws.TCoefficients.has_C2 {
                settings.tcoefficients.C[1] = ws.TCoefficients.C2;
                need_write = true;
            }

            if ws.TCoefficients.has_C3 {
                settings.tcoefficients.C[2] = ws.TCoefficients.C3;
                need_write = true;
            }

            if ws.TCoefficients.has_C4 {
                settings.tcoefficients.C[3] = ws.TCoefficients.C4;
                need_write = true;
            }

            if ws.TCoefficients.has_C5 {
                settings.tcoefficients.C[4] = ws.TCoefficients.C5;
                need_write = true;
            }
        }

        Ok(())
    })?;

    if need_write {
        start_writing_settings().map_err(|e| crate::settings::SettingActionError::AccessError(e))
    } else {
        Ok(())
    }
}

fn start_writing_settings() -> Result<(), FreeRtosError> {
    use freertos_rust::{Task, TaskPriority};
    defmt::warn!("Save settings rquested...");

    Task::new()
        .name("SS")
        .stack_size(384)
        .priority(TaskPriority(1))
        .start(move |_| {
            if let Err(e) = crate::settings::settings_save(Duration::infinite()) {
                defmt::error!("Failed to store settings: {}", defmt::Debug2Format(&e));
            }
        })
        .map(|_| ())?;

    Ok(())
}
