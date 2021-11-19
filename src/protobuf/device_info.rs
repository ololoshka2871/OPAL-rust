use my_proc_macro::git_version;

use super::messages::ru_sktbelpa_pressure_self_writer_InfoResponse;

pub fn fill_info(info: &mut ru_sktbelpa_pressure_self_writer_InfoResponse) -> Result<(), ()> {
    info.HW_Version = 1; // TODO
    info.SW_Version = git_version!();

    info.PressureChannelFailed = false; // TODO
    info.TemperatureChannelFailed = false; // TODO

    info.OverpressDetected = false; // TODO
    info.OverheatDetected = false; // TODO
    info.OverheatCPUDetected = false; // TODO

    Ok(())
}
