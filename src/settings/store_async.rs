use freertos_rust::{Duration, FreeRtosError, Mutex, Timer};
use lazy_static::lazy_static;

pub fn start_writing_settings(realy_write: bool) -> Result<(), FreeRtosError> {
    lazy_static! {
        static ref TIMER: Mutex<Option<Timer>> = crate::support::new_global_mutex();
    };

    if !realy_write {
        defmt::warn!("Save settings skipped...");
        return Ok(());
    }

    defmt::warn!("Save settings rquested...");

    let saver = move |_| {
        if let Err(e) = crate::settings::settings_save(Duration::infinite()) {
            defmt::error!("Failed to store settings: {}", defmt::Debug2Format(&e));
        }

        let _ = TIMER
            .lock(Duration::infinite())
            .map(|mut guard| *guard = None);
    };

    let timer = Timer::new(Duration::ms(1))
        .set_name("SS")
        .set_auto_reload(false)
        .create(saver)?;

    TIMER.lock(Duration::infinite()).map(|mut guard| {
        let _ = timer.start(Duration::infinite());
        *guard = Some(timer);
    })?;

    Ok(())
}
