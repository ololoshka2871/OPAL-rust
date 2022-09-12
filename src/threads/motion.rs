use alloc::sync::Arc;

use freertos_rust::{Duration, Mutex, Queue};
use usbd_serial::SerialPort;

use crate::gcode::{GCode, MotionMGR, MotionStatus, Request};

use crate::support::defmt_string::DefmtString;

use crate::threads::gcode_server::write_responce;

pub fn motion<B, LASER, GALVO>(
    serial: Arc<Mutex<&'static mut SerialPort<B>>>,
    gcode_queue: Arc<Queue<GCode>>,
    request_queue: Arc<Queue<Request>>,
    laser: LASER,
    galvo: GALVO,
    master_freq: stm32f1xx_hal::time::Hertz,
) -> !
where
    B: usb_device::bus::UsbBus,
    GALVO: crate::control::xy2_100::XY2_100Interface,
    LASER: crate::control::laser::LaserInterface,
{
    let master = crate::time_base::master_counter::MasterCounter::acquire();

    let mut motion = MotionMGR::new(
        galvo,
        laser,
        master,
        1_000_000_000f32 / master_freq.to_Hz() as f32,
    );
    motion.begin();
    loop {
        let mut cmd_got = false;
        if motion.tic() == MotionStatus::IDLE {
            if let Ok(gcode) = gcode_queue.receive(Duration::zero()) {
                cmd_got = true;
                match motion.process(&gcode) {
                    Ok(Some(msg)) => {
                        write_responce(&serial, msg.as_str());
                    }
                    Ok(None) => {
                        write_responce(&serial, "ok\n");
                    }
                    Err(e) => {
                        write_responce(&serial, "error\n");
                        defmt::error!("Failed to process command {}", DefmtString(&e));
                    }
                }
            }
        } else {
            cmd_got = true;
        }

        if let Ok(req) = request_queue.receive(Duration::zero()) {
            cmd_got = true;
            match motion.process_status_req(&req) {
                Ok(Some(msg)) => {
                    write_responce(&serial, msg.as_str());
                }
                Ok(None) => {
                    write_responce(&serial, "ok\n");
                }
                Err(e) => {
                    write_responce(&serial, "error\n");
                    defmt::error!("Failed to process command {}", DefmtString(&e));
                }
            }
        }

        if !cmd_got {
            crate::support::mast_yield();
        }
    }
}
