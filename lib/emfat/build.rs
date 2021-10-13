fn main() {
    let src = [
        "emfat-src/emfat.c",
    ];
    let mut builder = cc::Build::new();
    let build = builder
        .files(src.iter())
        .include("emfat-src")
        .flag("-Wno-unused-parameter")
        .flag("-fno-aggressive-loop-optimizations")
        //.define("SOME_MACRO", Some("0"))
        ;
    build.compile("emfat");
}