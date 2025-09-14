// build.rs
use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Tell cargo to recompile if the proto file changes
    println!("cargo:rerun-if-changed=proto/market_data.proto");

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("market_data_descriptor.bin"))
        .build_server(true)
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["proto/market_data.proto"], &["proto"])?;

    Ok(())
}
