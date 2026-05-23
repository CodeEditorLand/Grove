#![allow(non_snake_case)]

//! Build script for Grove
//!
//! This script handles:
//! - Proto file compilation with tonic-build
//! - WASM-specific build configuration
//! - Feature-based conditional compilation
//! - Build timestamp generation with vergen

use std::{env, path::PathBuf};

fn main() -> anyhow::Result<()> {
	// vergen 9 replaced the 8.x `EmitBuilder` facade with per-domain builders
	// (`BuildBuilder`, `CargoBuilder`, `RustcBuilder`) fed into a shared
	// `Emitter`. Same `VERGEN_*` env vars are emitted so downstream
	// `env!("VERGEN_…")` reads in Grove continue to work unchanged.
	let BuildInstructions = vergen::BuildBuilder::all_build()?;

	let CargoInstructions = vergen::CargoBuilder::all_cargo()?;

	let RustcInstructions = vergen::RustcBuilder::all_rustc()?;

	vergen::Emitter::default()
		.add_instructions(&BuildInstructions)?
		.add_instructions(&CargoInstructions)?
		.add_instructions(&RustcInstructions)?
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

	// tonic-build 0.14 split the `configure()` → `Builder` → `compile_protos`
	// flow into `tonic-prost-build`. Same fluent surface (`build_server`,
	// `build_client`, `out_dir`, `compile_protos`) - just a different crate.
	tonic_prost_build::configure()
		.build_server(true)
		.build_client(true)
		.out_dir(&out_dir)
		.compile_protos(&[proto_file.as_path()], &[proto_dir.as_path()])
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
