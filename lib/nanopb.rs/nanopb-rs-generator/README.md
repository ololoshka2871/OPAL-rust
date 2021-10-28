# Support library for code generation

## Usage

1. Add dependency
```ini
[build-dependencies]
cc = "1.0.52"
nanopb-rs-generator = { path = "lib/nanopb.rs/nanopb-rs-generator" }
```

2. Create .proto file
3. Add to `build.rs`
```rust
let mut cc = cc::Build::new();
let protobuf_src = nanopb_rs_generator::Generator::new()
        .add_proto_file("src/ProtobufDevice_0000E006.proto")
        .generate();

cc.file(protobuf_src).include("lib/nanopb.rs/nanopb-dist");
cc.try_compile("protobuf-proto").unwrap_or_else(|e| panic!("{}", e.to_string()));
```