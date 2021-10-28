fn main() {
    let src = [
        "../nanopb-dist/pb_common.c",
        "../nanopb-dist/pb_encode.c",
        "../nanopb-dist/pb_decode.c",
    ];
    let mut builder = cc::Build::new();
    let build = builder
        .files(src.iter())
        .include("../nanopb-dist")
        //.flag("-Wno-unused-parameter")
        //.flag("-fno-aggressive-loop-optimizations")
        //.define("SOME_MACRO", Some("0"))
        ;
    build.compile("nanopb-core");
}