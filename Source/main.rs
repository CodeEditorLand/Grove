//! Grove Standalone Binary
//!
//! This is the entry point for running Grove as a standalone extension host.
//! It can operate independently or connect to Mountain via gRPC.

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use Grove::{
	Binary::{
		Build::{RuntimeBuild, ServiceRegister},
		Main::Entry::{BuildResult, Entry, ExtensionInfo, ValidationResult},
	},
	Transport::{
		IPCTransport::IPCTransport,
		Strategy::Transport as TransportEnum,
		WASMTransport::WASMTransportImpl,
		gRPCTransport::gRPCTransport,
	},
	dev_log,
};

/// Grove - Rust/WASM Extension Host for VS Code
///
/// Grove provides a secure, sandboxed environment for running VS Code
/// extensions compiled to WebAssembly or native Rust.
#[derive(Parser, Debug)]
#[command(name = "grove")]
#[command(author = "Grove Contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Rust/WASM Extension Host for VS Code", long_about = None)]
struct Cli {
	/// Mode of operation
	#[command(subcommand)]
	mode:Mode,

	/// Verbosity level (-v, -vv, -vvv)
	#[arg(short, long, action = clap::ArgAction::Count)]
	verbose:u8,

	/// Log format (plain, json)
	#[arg(long, default_value = "plain")]
	log_format:String,
}

/// Grove operation modes
#[derive(Subcommand, Debug)]
enum Mode {
	/// Run Grove in standalone mode
	Standalone {
		/// Path to extension directory or manifest
		#[arg(short, long)]
		extension:Option<PathBuf>,

		/// Transport type (grpc, ipc, wasm)
		#[arg(short, long, default_value = "wcs")]
		transport:String,

		/// Listen address for gRPC server
		#[arg(long, default_value = "127.0.0.1:50051")]
		grpc_address:String,

		/// Enable WASM WASI support
		#[arg(long)]
		wasi:bool,

		/// Memory limit in MB for WASM modules
		#[arg(long, default_value = "512")]
		memory_limit_mb:u64,

		/// Maximum execution time in milliseconds
		#[arg(long, default_value = "30000")]
		max_execution_time_ms:u64,
	},

	/// Run Grove as a service connected to Mountain
	Service {
		/// Mountain gRPC address
		#[arg(short, long, default_value = "127.0.0.1:50050")]
		mountain_address:String,

		/// Service name identification
		#[arg(long)]
		service_name:Option<String>,

		/// Enable auto-reconnect
		#[arg(long)]
		auto_reconnect:bool,
	},

	/// Load and validate an extension without activating
	Validate {
		/// Path to extension manifest
		manifest_path:PathBuf,

		/// Detailed validation output
		#[arg(short, long)]
		detailed:bool,
	},

	/// Build WASM module from Rust source
	Build {
		/// Source directory
		#[arg(short, long)]
		source:PathBuf,

		/// Output path for WASM module
		#[arg(short, long)]
		output:PathBuf,

		/// Optimization level (0-3, s, z)
		#[arg(short, long, default_value = "3")]
		opt_level:String,

		/// Target triple (e.g., wasm32-wasi)
		#[arg(long)]
		target:Option<String>,
	},

	/// List all loaded extensions
	List {
		/// Show detailed information
		#[arg(short, long)]
		detailed:bool,
	},
}

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();

	// [Boot] [Telemetry] Bring up shared dual-pipe (PostHog + OTLP). No-op
	// in release builds and when `Capture=false`.
	CommonLibrary::Telemetry::Initialize::Fn(CommonLibrary::Telemetry::Tier::Tier::Grove).await;

	// Initialize logging
	init_logging(cli.verbose, &cli.log_format)?;

	// Start based on mode
	match cli.mode {
		Mode::Standalone { extension, transport, grpc_address, wasi, memory_limit_mb, max_execution_time_ms } => {
			run_standalone(extension, transport, grpc_address, wasi, memory_limit_mb, max_execution_time_ms).await
		},

		Mode::Service { mountain_address, service_name, auto_reconnect } => {
			run_service(mountain_address, service_name, auto_reconnect).await
		},

		Mode::Validate { manifest_path, detailed } => run_validation(manifest_path, detailed).await,

		Mode::Build { source, output, opt_level, target } => run_build(source, output, opt_level, target).await,

		Mode::List { detailed } => run_list(detailed).await,
	}
}

/// Initialize logging with appropriate level and format
fn init_logging(_verbose:u8, _format:&str) -> Result<()> { Ok(()) }

/// Run Grove in standalone mode
async fn run_standalone(
	extension:Option<PathBuf>,

	transport_type:String,

	grpc_address:String,

	wasi:bool,

	memory_limit_mb:u64,

	max_execution_time_ms:u64,
) -> Result<()> {
	dev_log!("grove", "Starting Grove in standalone mode...");

	let transport = match transport_type.as_str() {
		"grpc" => TransportEnum::gRPC(gRPCTransport::New(&grpc_address)?),

		"ipc" => TransportEnum::IPC(IPCTransport::New()?),

		"wasm" => TransportEnum::WASM(WASMTransportImpl::new(wasi, memory_limit_mb, max_execution_time_ms)?),

		_ => TransportEnum::default(),
	};

	dev_log!("transport", "Using transport: {:?}", transport_type);

	let host =
		RuntimeBuild::RuntimeBuild::build_host_with_defaults(transport, wasi, memory_limit_mb, max_execution_time_ms)
			.await?;

	if let Some(path) = extension {
		dev_log!("extensions", "Loading extension from: {:?}", path);

		let ext_id = host.load_extension(&path).await.map_err(|e| {
			dev_log!("extensions", "error: failed to load extension: {}", e);

			e
		})?;

		dev_log!("extensions", "Extension loaded successfully with ID: {}", ext_id);

		host.activate(&ext_id).await?;

		dev_log!("extensions", "Extension activated");
	} else {
		dev_log!("grove", "No extension specified, running in daemon mode");

		keep_running().await;
	}

	Ok(())
}

/// Run Grove as a service connected to Mountain
async fn run_service(mountain_address:String, service_name:Option<String>, auto_reconnect:bool) -> Result<()> {
	dev_log!("grove", "Starting Grove as service...");

	let name = service_name.unwrap_or_else(|| "grove-host".to_string());

	dev_log!("grove", "Service name: {}", name);

	dev_log!("grove", "Mountain address: {}", mountain_address);

	// Register with Mountain
	ServiceRegister::ServiceRegister::register_with_mountain(&name, &mountain_address, auto_reconnect).await?;

	keep_running().await;

	Ok(())
}

/// Validate an extension manifest
async fn run_validation(manifest_path:PathBuf, detailed:bool) -> Result<()> {
	dev_log!("extensions", "Validating extension manifest: {:?}", manifest_path);

	let result:ValidationResult = Entry::validate_extension(&manifest_path, detailed).await?;

	if result.is_valid {
		dev_log!("extensions", "Extension manifest is valid");

		if detailed {
			println!("{:#?}", result);
		}
	} else {
		dev_log!("extensions", "error: extension manifest validation failed");

		return Err(anyhow::anyhow!("Validation failed"));
	}

	Ok(())
}

/// Build a WASM module from Rust source
async fn run_build(source:PathBuf, output:PathBuf, opt_level:String, target:Option<String>) -> Result<()> {
	dev_log!("wasm", "Building WASM module from: {:?}", source);

	dev_log!("wasm", "Output path: {:?}", output);

	dev_log!("wasm", "Optimization level: {}", opt_level);

	let result:BuildResult = Entry::build_wasm_module(source, output, opt_level, target).await?;

	if result.success() {
		dev_log!("wasm", "WASM module built successfully");
	} else {
		dev_log!("wasm", "error: WASM module build failed");

		return Err(anyhow::anyhow!("Build failed"));
	}

	Ok(())
}

/// List all loaded extensions
async fn run_list(detailed:bool) -> Result<()> {
	dev_log!("extensions", "Listing extensions...");

	let extensions:Vec<ExtensionInfo> = Entry::list_extensions(detailed).await?;

	if extensions.is_empty() {
		dev_log!("extensions", "No extensions loaded");
	} else {
		println!("Loaded extensions:");

		for ext in extensions {
			if detailed {
				println!("  {:#?}", ext);
			} else {
				println!("  {} ({})", ext.name, ext.version);
			}
		}
	}

	Ok(())
}

/// Keep the process running
async fn keep_running() {
	dev_log!("lifecycle", "Grove is running. Press Ctrl+C to stop.");

	tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");

	dev_log!("lifecycle", "Shutting down Grove...");
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_cli_parsing() {
		let cli = Cli::try_parse_from(["grove", "standalone", "--extension", "/tmp/ext", "--transport", "wasm"]);

		assert!(cli.is_ok());
	}

	#[test]
	fn test_logging_levels() {
		// Test that logging can be initialized at different levels
		let _ = init_logging(0, "plain");

		let _ = init_logging(1, "plain");

		let _ = init_logging(2, "plain");

		let _ = init_logging(3, "plain");
	}
}
