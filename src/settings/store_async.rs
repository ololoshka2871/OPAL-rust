use freertos_rust::{Duration, FreeRtosError, Mutex, Timer};
use lazy_static::lazy_static;

pub fn start_writing_settings(realy_write: bool) -> Result<(), FreeRtosError> {
    lazy_static! {
        static ref TIMER: Mutex<Option<Timer>> = Mutex::new(None).unwrap();
    };

    if !realy_write {
        defmt::warn!("Save settings skipped...");
        return Ok(());
    }

    defmt::warn!("Save settings rquested...");

    let saver = |_| {
        if let Err(e) = crate::settings::settings_save(Duration::infinite()) {
            defmt::error!("Failed to store settings: {}", defmt::Debug2Format(&e));
        }

        TIMER
            .lock(Duration::infinite())
            .map(|mut guard| *guard = None)
            .unwrap();
    };

    let timer = Timer::new(Duration::ms(1))
        .set_name("SS")
        .set_auto_reload(false)
        .create(saver)?;

    TIMER.lock(Duration::infinite()).map(|mut guard| {
        timer.start(Duration::infinite()).unwrap();
        *guard = Some(timer);
    })?;

    Ok(())
}
