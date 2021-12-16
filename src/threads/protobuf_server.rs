use alloc::sync::Arc;

use freertos_rust::{CurrentTask, Duration, Mutex};

use nanopb_rs::{Error, IStream};

use usbd_serial::SerialPort;

use crate::{protobuf, workmodes::output_storage::OutputStorage};

pub fn protobuf_server<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
    output: Arc<Mutex<OutputStorage>>,
) -> ! {
    loop {
        let msg_size =
            match protobuf::recive_md_header1(&mut new_istream(serial_container.clone(), None)) {
                Ok(size) => size,
                Err(e) => {
                    print_error(e);
                    continue;
                }
            };

        let request = match protobuf::recive_message_body1(new_istream(
            serial_container.clone(),
            Some(msg_size),
        )) {
            Ok(request) => request,
            Err(e) => {
                print_error(e);
                continue;
            }
        };

        //defmt::info!("Nanopb: Request:\n{}", defmt::Debug2Format(&request));
        defmt::info!("Nanopb: Request id={}", request.id);

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

        //defmt::info!("Nanopb: Response:\n{}", defmt::Debug2Format(&response));
        defmt::info!("Nanopb: Response ready");

        if let Err(e) = write_responce(serial_container.clone(), response) {
            print_error(e);
        }
    }
}

fn print_error(e: Error) {
    defmt::error!("Protobuf error: {}", defmt::Display2Format(&e));
}

fn write_responce<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
    response: protobuf::ru_sktbelpa_pressure_self_writer_Response,
) -> Result<(), Error> {
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

fn new_istream<B: usb_device::bus::UsbBus>(
    serial_container: Arc<Mutex<SerialPort<B>>>,
    stream_size: Option<usize>,
) -> IStream<protobuf::Reader<B>> {
    IStream::from_callback(
        protobuf::Reader {
            container: serial_container,
        },
        stream_size,
    )
}
