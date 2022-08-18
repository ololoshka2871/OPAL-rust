use alloc::sync::Arc;
use freertos_rust::{Duration, Queue};

use crate::gcode::GCode;

pub fn motion(gcode_queue: Arc<Queue<GCode>>) -> ! {
    // motion = new MotionMGR(&commandBuffer);
    // motion->begin(galvo, laser);
    loop {
        if let Ok(gcode) = gcode_queue.receive(Duration::zero()) {
            defmt::info!("New commad: {}", gcode);
        }
        //motion->tic();
    }
}
