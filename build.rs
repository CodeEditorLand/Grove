//! Build script for Grove
//!
//! This script handles:
//! - Proto file compilation with tonic-build
//! - WASM-specific build configuration
//! - Feature-based conditional compilation
//! - Build timestamp generation with vergen

use std::env;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    // Generate build timestamp and other vergen environment variables
    vergen::EmitBuilder::builder()
        .all_build()
        .all_cargo()
        .all_git()
        .all_rustc()
        .emit()?;
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_dir = manifest_dir.join("Proto");
    
    println!("cargo:rerun-if-changed={}", proto_dir.join("Grove.proto").display());
    println!("cargo:rerun-if-changed={}", proto_dir.display());

    // Detect if we're building for WASM
    let is_wasm = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32";
    
    // Check if grpc feature is enabled
    let grpc_enabled = env::var("CARGO_FEATURE_GRPC").is_ok();

    if !is_wasm && grpc_enabled {
        // Only compile protos for native builds with grpc feature
        compile_protos();
    } else {
        if is_wasm {
            println!("cargo:warning=Building for WASM target, skipping gRPC proto compilation");
        }
        if !grpc_enabled {
            println!("cargo:warning=grpc feature not enabled, skipping gRPC proto compilation");
        }
    }

    // WASM-specific configuration
    configure_wasm_build();

    Ok(())
}

fn compile_protos() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_dir = manifest_dir.join("Proto");
    let proto_file = proto_dir.join("Grove.proto");
    let out_dir = manifest_dir.join("Source/Protocol/Generated");
    
    // Create the output directory if it doesn't exist
    std::fs::create_dir_all(&out_dir).expect(&format!("Failed to create directory: {:?}", out_dir));
    
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(&out_dir)
        .compile(&[proto_file], &[proto_dir])
        .expect("Failed to compile protos");

    println!("cargo:rerun-if-changed={}", out_dir.display());
}

fn configure_wasm_build() {
    // Set linker flags for WASM
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32" {
        // Optimize for smaller WASM binaries
        println!("cargo:rustc-cfg=wasm32");
        println!("cargo:rustc-cfg=web_sys_unstable_apis");
    }
}
