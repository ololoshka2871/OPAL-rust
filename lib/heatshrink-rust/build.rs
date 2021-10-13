fn main() {
    let src = [
        "heatshrink/heatshrink_decoder.c",
        "heatshrink/heatshrink_encoder.c",
    ];
    let mut builder = cc::Build::new();
    let build = builder
        .files(src.iter())
        .include("heatshrink")
        .flag("-Wno-implicit-fallthrough")
        .define("HEATSHRINK_DYNAMIC_ALLOC", Some("0"))
        //.define("HEATSHRINK_DEBUGGING_LOGS", Some("1"))
        ;
    build.compile("heatshrink");
}