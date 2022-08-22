use alloc::sync::Arc;
use freertos_rust::{Duration, Queue};

use crate::gcode::{GCode, MotionMGR, MotionStatus};

pub fn motion(
    gcode_queue: Arc<Queue<GCode>>,
    laser: u32,
    galvo: crate::control::xy2_100::XY2_100,
    master_freq: stm32l4xx_hal::time::Hertz,
) -> ! {
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
