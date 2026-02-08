//! Build script for Grove
//!
//! This script handles:
//! - Proto file compilation with tonic-build
//! - WASM-specific build configuration
//! - Feature-based conditional compilation

use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=Proto/Grove.proto");
    println!("cargo:rerun-if-changed=Proto/");

    // Detect if we're building for WASM
    let is_wasm = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32";

    if !is_wasm {
        // Only compile protos for native builds
        compile_protos();
    } else {
        println!("cargo:warning=Building for WASM target, skipping gRPC proto compilation");
    }

    // WASM-specific configuration
    configure_wasm_build();

    Ok(())
}

fn compile_protos() {
    let proto_files = vec!["Proto/Grove.proto"];
    let includes = vec!["Proto/"];

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("Source/Protocol/Generated")
        .compile_files(&proto_files.iter().map(PathBuf::from).collect::<Vec<_>>(), &includes)
        .expect("Failed to compile protos");

    println!("cargo:rerun-if-changed=Source/Protocol/Generated/");
}

fn configure_wasm_build() {
    // Set linker flags for WASM
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32" {
        // Optimize for smaller WASM binaries
        println!("cargo:rustc-cfg=wasm32");
        println!("cargo:rustc-cfg=web_sys_unstable_apis");
    }
}
