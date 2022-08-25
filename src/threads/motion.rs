use core::convert::Infallible;

use alloc::sync::Arc;
use embedded_hal::PwmPin;
use freertos_rust::{Duration, Mutex, Queue};
use stm32l4xx_hal::prelude::OutputPin;
use usbd_serial::SerialPort;

use crate::gcode::{GCode, MotionMGR, MotionStatus, Request};

use crate::threads::gcode_server::write_responce;

pub fn motion<B, PWM, ENABLE, GALVOEN>(
    serial: Arc<Mutex<&'static mut SerialPort<B>>>,
    gcode_queue: Arc<Queue<GCode>>,
    request_queue: Arc<Queue<Request>>,
    laser: crate::control::laser::Laser<PWM, ENABLE>,
    galvo: crate::control::xy2_100::XY2_100<GALVOEN>,
    master_freq: stm32l4xx_hal::time::Hertz,
) -> !
where
    B: usb_device::bus::UsbBus,
    PWM: PwmPin<Duty = u16>,
    ENABLE: OutputPin<Error = Infallible>,
    GALVOEN: OutputPin<Error = Infallible>,
{
    let master = crate::time_base::master_counter::MasterCounter::acquire();

    let mut motion = MotionMGR::new(
        laser,
        galvo,
        master,
        1_000_000_000f64 / master_freq.to_Hz() as f64,
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
                        defmt::error!("Failed to process command {}", defmt::Display2Format(&e));
                    }
                }
            }
        } else {
            cmd_got = true;
        }

        if let Ok(req) = request_queue.receive(Duration::zero()) {
            cmd_got = true;
            match motion.process_req(&req) {
                Ok(Some(msg)) => {
                    write_responce(&serial, msg.as_str());
                }
                Ok(None) => {
                    write_responce(&serial, "ok\n");
                }
                Err(e) => {
                    write_responce(&serial, "error\n");
                    defmt::error!("Failed to process command {}", defmt::Display2Format(&e));
                }
            }
        }

        if !cmd_got {
            crate::support::mast_yield();
        }
    }
}
