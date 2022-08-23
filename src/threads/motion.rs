use core::convert::Infallible;

use alloc::sync::Arc;
use embedded_hal::PwmPin;
use freertos_rust::{Duration, Queue};
use stm32l4xx_hal::prelude::OutputPin;

use crate::gcode::{GCode, MotionMGR, MotionStatus};

pub fn motion<PWM, ENABLE, GALVOEN>(
    gcode_queue: Arc<Queue<GCode>>,
    laser: crate::control::laser::Laser<PWM, ENABLE>,
    galvo: crate::control::xy2_100::XY2_100<GALVOEN>,
    master_freq: stm32l4xx_hal::time::Hertz,
) -> !
where
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
        if motion.tic() == MotionStatus::IDLE {
            if let Ok(gcode) = gcode_queue.receive(Duration::zero()) {
                if let Err(e) = motion.process(gcode) {
                    defmt::error!(
                        "Failed to process command {}\n> {}",
                        gcode,
                        defmt::Display2Format(&e)
                    );
                } else {
                    defmt::info!("New commad: {}", gcode);
                }
            }
        }
    }
}
