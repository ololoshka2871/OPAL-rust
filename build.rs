include!("src/config.rs");

use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn generate_free_rtos_config<P: AsRef<Path>>(path: P) -> PathBuf {
    let outpath = PathBuf::from(env::var("OUT_DIR").unwrap());

    let config_file = "FreeRTOSConfig.h";
    let mut infile = path.as_ref().to_path_buf();
    infile.push(config_file);

    let cfg = fs::read_to_string(infile.clone())
        .expect(format!("Failed to read {}", infile.to_str().unwrap()).as_str());

    let out_cfg = cfg
        .replace(
            "%RUNTIME_STATS%",
            if cfg!(debug_assertions) { "1" } else { "0" },
        )
        .replace("%F_CPU%", format!("{}UL", FREERTOS_CONFIG_FREQ).as_str());

    let mut out_file = outpath.clone();
    out_file.push(config_file);
    fs::write(out_file.clone(), out_cfg)
        .expect(format!("Failed to write {}", out_file.to_str().unwrap()).as_str());

    outpath
}

fn build_freertos(mut b: freertos_cargo_build::Builder) {
    // Path to FreeRTOS kernel or set ENV "FREERTOS_SRC" instead
    b.freertos("./FreeRTOS-Kernel");
    b.freertos_port(String::from("GCC/ARM_CM4F")); // Port dir relativ to 'FreeRTOS-Kernel/portable'

    b.freertos_config(&generate_free_rtos_config("src/configTemplate"));

    /*
    // Location of `FreeRTOSConfig.h`
    if cfg!(debug_assertions) {
        b.freertos_config("src/configDebug");
    } else {
        b.freertos_config("src/configRelease");
    }
    */

    // выбор не работает
    // b.heap(String::from("heap4.c")); // Set the heap_?.c allocator to use from
    // 'FreeRTOS-Kernel/portable/MemMang' (Default: heap_4.c)

    // другие "С"-файлы
    // b.get_cc().file("More.c");   // Optional additional C-Code to be compiled
    b.compile().unwrap_or_else(|e| panic!("{}", e.to_string()));
}

fn main() {
    build_freertos(freertos_cargo_build::Builder::new());
}
