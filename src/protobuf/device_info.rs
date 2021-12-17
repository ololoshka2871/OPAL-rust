use my_proc_macro::git_version;

pub fn fill_info(info: &mut super::messages::InfoResponse) -> Result<(), ()> {
    info.hw_version = 1; // TODO
    info.sw_version = git_version!();

    info.pressure_channel_failed = false; // TODO
    info.temperature_channel_failed = false; // TODO

    info.overpress_detected = false; // TODO
    info.overheat_detected = false; // TODO
    info.overheat_cpu_detected = false; // TODO
    info.over_vbat_detected = false; // TODO

    Ok(())
}
