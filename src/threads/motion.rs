use alloc::sync::Arc;
use freertos_rust::{Duration, Queue};

use crate::gcode::{GCode, MotionMGR, MotionStatus};

pub fn motion(
    gcode_queue: Arc<Queue<GCode>>,
    laser: u32,
    galvo: crate::control::xy2_100::xy2_100,
) -> ! {
    let mut motion = MotionMGR::new(laser, galvo);
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
