//! Entry Module (Binary/Main)
//!
//! Main entry point for the Grove binary.
//! Handles CLI argument parsing and initialization of the Grove host.

use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing::{error, info, instrument};

use crate::{
	Binary::Build::ServiceRegister,
	Binary::Main::CliArgs,
	Host::{HostConfig, ExtensionHost::ExtensionHostImpl},
	Transport::Transport,
	WASM::Runtime::{WASMConfig, WASMRuntime},
};

/// Grove entry point manager
pub struct Entry;

impl Entry {
	/// Main entry point for the Grove binary
	#[instrument(skip(args))]
	pub async fn run(args:CliArgs) -> Result<()> {
		info!("Starting Grove v{}", env!("CARGO_PKG_VERSION"));
		info!("Mode: {}", args.mode);

		match args.mode.as_str() {
			"standalone" => Self::run_standalone(args).await,
			"service" => Self::run_service(args).await,
			"validate" => Self::run_validation(args).await,
			_ => Err(anyhow::anyhow!("Unknown mode: {}", args.mode)),
		}
	}

	/// Run in standalone mode
	#[instrument(skip(args))]
	async fn run_standalone(args:CliArgs) -> Result<()> {
		info!("Starting Grove in standalone mode");

		// Create transport
		let transport = Self::create_transport(&args)?;

		// Create host configuration
		let host_config = HostConfig::default().with_activation_timeout(args.max_execution_time_ms);

		// Create extension host
		let host = ExtensionHostImpl::with_config(transport, host_config)
			.await
			.context("Failed to create extension host")?;

		// Load and activate extension if specified
		if let Some(extension_path) = args.extension {
			let path = PathBuf::from(extension_path);
			host.load_extension(&path).await?;
			host.activate_all().await?;
		} else {
			info!("No extension specified, running in daemon mode");
		}

		// Keep running until interrupted
		Self::wait_for_shutdown().await;

		// Shutdown host
		host.shutdown().await?;

		Ok(())
	}

	/// Run as a service
	#[instrument(skip(args))]
	async fn run_service(args:CliArgs) -> Result<()> {
		info!("Starting Grove as service");

		// Create transport for Mountain communication
		let transport = Transport::default();

		// Register with Mountain
		#[cfg(feature = "gRPC")]
		{
			match crate::Binary::Build::ServiceRegister::register_with_mountain(
				"grove-host",
				&args.mountain_address,
				true, // auto reconnect
			).await {
				Ok(_) => info!("Registered with Mountain"),
				Err(e) => warn!("Failed to register with Mountain: {}", e),
			}
		}

		#[cfg(not(feature = "gRPC"))]
		{
			info!("gRPC feature not enabled, skipping Mountain registration");
		}

		// Keep running
		Self::wait_for_shutdown().await;

		Ok(())
	}

	/// Validate an extension
	#[instrument(skip(args))]
	async fn run_validation(args:CliArgs) -> Result<()> {
		info!("Validating extension");

		let extension_path = args
			.extension
			.ok_or_else(|| anyhow::anyhow!("Extension path required for validation"))?;

		let path = PathBuf::from(extension_path);
		let result = Self::validate_extension(&path, false).await?;

		if result.is_valid {
			info!("Extension validation passed");
			Ok(())
		} else {
			error!("Extension validation failed");
			Err(anyhow::anyhow!("Validation failed"))
		}
	}

	/// Validate an extension manifest
	pub async fn validate_extension(path:&PathBuf, detailed:bool) -> Result<ValidationResult> {
		info!("Validating extension at: {:?}", path);

		// Check if path exists
		if !path.exists() {
			return Ok(ValidationResult { is_valid:false, errors:vec![format!("Path does not exist: {:?}", path)] });
		}

		let mut errors = Vec::new();

		// Parse package.json
		let package_json_path = path.join("package.json");
		if package_json_path.exists() {
			match tokio::fs::read_to_string(&package_json_path).await {
				Ok(content) => {
					match serde_json::from_str::<serde_json::Value>(&content) {
						Ok(_) => {
							info!("Valid package.json found");
						},
						Err(e) => {
							errors.push(format!("Invalid package.json: {}", e));
						},
					}
				},
				Err(e) => {
					errors.push(format!("Failed to read package.json: {}", e));
				},
			}
		} else {
			errors.push("package.json not found".to_string());
		}

		let is_valid = errors.is_empty();

		if detailed && !errors.is_empty() {
			for error in &errors {
				info!("Validation error: {}", error);
			}
		}

		Ok(ValidationResult { is_valid, errors })
	}

	/// Build a WASM module
	pub async fn build_wasm_module(
		source:PathBuf,
		output:PathBuf,
		opt_level:String,
		target:Option<String>,
	) -> Result<BuildResult> {
		info!("Building WASM module from: {:?}", source);
		info!("Output: {:?}", output);
		info!("Optimization level: {}", opt_level);

		// For now, return a placeholder result
		// In production, this would invoke rustc/cargo with wasm32-wasi target
		Ok(BuildResult { success:true, output_path:output, compile_time_ms:0 })
	}

	/// List loaded extensions
	pub async fn list_extensions(detailed:bool) -> Result<Vec<ExtensionInfo>> {
		info!("Listing extensions");

		// For now, return empty list
		// In production, this would query the extension manager
		Ok(Vec::new())
	}

	/// Create transport based on arguments
	fn create_transport(args:&CliArgs) -> Result<Transport> {
		match args.transport.as_str() {
			"grpc" => {
				use crate::Transport::gRPCTransport::GrpcTransport;
				Ok(Transport::gRPC(
					GrpcTransport::new(&args.grpc_address)
						.context("Failed to create gRPC transport")?,
				))
			},
			"ipc" => {
				use crate::Transport::IPCTransport::IPCTransportImpl;
				Ok(Transport::IPC(
					IPCTransportImpl::new().context("Failed to create IPC transport")?,
				))
			},
			"wasm" => {
				use crate::Transport::WASMTransport::WASMTransportImpl;
				Ok(Transport::WASM(
					WASMTransportImpl::new(args.wasi, args.memory_limit_mb, args.max_execution_time_ms)
						.context("Failed to create WASM transport")?,
				))
			},
			_ => Ok(Transport::default()),
		}
	}

	/// Wait for shutdown signal
	async fn wait_for_shutdown() {
		info!("Grove is running. Press Ctrl+C to stop.");

		tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");

		info!("Shutdown signal received");
	}
}

impl Default for Entry {
	fn default() -> Self { Self }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
	/// Whether validation passed
	pub is_valid:bool,
	/// Validation errors
	pub errors:Vec<String>,
}

/// Build result
#[derive(Debug, Clone)]
pub struct BuildResult {
	/// Whether build succeeded
	pub success:bool,
	/// Output path
	pub output_path:PathBuf,
	/// Compile time in ms
	pub compile_time_ms:u64,
}

impl BuildResult {
	/// Check if build succeeded
	pub fn success(&self) -> bool { self.success }
}

/// Extension info for listing
#[derive(Debug, Clone)]
pub struct ExtensionInfo {
	/// Extension ID
	pub name:String,
	/// Extension version
	pub version:String,
	/// Extension path
	pub path:PathBuf,
	/// Is active
	pub is_active:bool,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_entry_default() {
		let entry = Entry::default();
		// Just test that it can be created
		let _ = entry;
	}

	#[tokio::test]
	async fn test_validate_extension_nonexistent() {
		let result = Entry::validate_extension(&PathBuf::from("/nonexistent/path"), false)
			.await
			.unwrap();

		assert!(!result.is_valid);
		assert!(!result.errors.is_empty());
	}

	#[test]
	fn test_cli_args_default() {
		let args = CliArgs::default();
		assert_eq!(args.mode, "standalone");
		assert!(args.wasi);
	}

	#[test]
	fn test_build_result() {
		let result = BuildResult {
			success:true,
			output_path:PathBuf::from("/test/output.wasm"),
			compile_time_ms:1000,
		};

		assert!(result.success());
		assert_eq!(result.compile_time_ms, 1000);
	}
}
