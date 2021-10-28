use freertos_cargo_build;
use nanopb_rs_generator;

fn generate_protobuf_src() {
    let _res = nanopb_rs_generator::Generator::new()
        .add_proto_file("src/ProtobufDevice_0000E006.proto")
        .generate();
}

fn main() {
    generate_protobuf_src();

    let mut b = freertos_cargo_build::Builder::new();

    // Path to FreeRTOS kernel or set ENV "FREERTOS_SRC" instead
    b.freertos("./FreeRTOS-Kernel");
    b.freertos_port(String::from("GCC/ARM_CM4F")); // Port dir relativ to 'FreeRTOS-Kernel/portable'

    // Location of `FreeRTOSConfig.h`
    if cfg!(debug_assertions) {
        b.freertos_config("src/configDebug");
    } else {
        b.freertos_config("src/configRelease");
    }

    // выбор не работает
    //b.heap(String::from("heap4.c")); // Set the heap_?.c allocator to use from
    // 'FreeRTOS-Kernel/portable/MemMang' (Default: heap_4.c)

    // другие "С"-файлы
    // b.get_cc().file("More.c");   // Optional additional C-Code to be compiled

    b.compile().unwrap_or_else(|e| panic!("{}", e.to_string()));
}
