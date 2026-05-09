//! Entry Module (Binary/Main)
//!
//! Main entry point for the Grove binary.
//! Handles CLI argument parsing and initialization of the Grove host.

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::{
	Binary::Main::CliArgs,
	Host::{ExtensionHost::ExtensionHostImpl, HostConfig},
	Transport::Strategy::Transport,
	dev_log,
};

/// Grove entry point manager
pub struct Entry;

impl Entry {
	/// Main entry point for the Grove binary
	pub async fn run(args:CliArgs) -> Result<()> {
		dev_log!("lifecycle", "Starting Grove v{}", env!("CARGO_PKG_VERSION"));

		dev_log!("lifecycle", "Mode: {}", args.mode);

		match args.mode.as_str() {
			"standalone" => Self::run_standalone(args).await,

			"service" => Self::run_service(args).await,

			"validate" => Self::run_validation(args).await,

			_ => Err(anyhow::anyhow!("Unknown mode: {}", args.mode)),
		}
	}

	/// Run in standalone mode
	async fn run_standalone(args:CliArgs) -> Result<()> {
		dev_log!("grove", "Starting Grove in standalone mode");

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
			dev_log!("grove", "No extension specified, running in daemon mode");
		}

		// Keep running until interrupted
		Self::wait_for_shutdown().await;

		// Shutdown host
		host.shutdown().await?;

		Ok(())
	}

	/// Run as a service
	async fn run_service(_args:CliArgs) -> Result<()> {
		dev_log!("grove", "Starting Grove as service");

		// Create transport for Mountain communication
		let _transport = Transport::default();

		// Register with Mountain
		#[cfg(feature = "gRPC")]
		{
			match crate::Binary::Build::ServiceRegister::register_with_mountain(
				"grove-host",
				&args.mountain_address,
				true, // auto reconnect
			)
			.await
			{
				Ok(_) => dev_log!("grove", "Registered with Mountain"),

				Err(e) => dev_log!("grove", "warn: failed to register with Mountain: {}", e),
			}
		}

		#[cfg(not(feature = "gRPC"))]
		{
			dev_log!("grpc", "gRPC feature not enabled, skipping Mountain registration");
		}

		// Keep running
		Self::wait_for_shutdown().await;

		Ok(())
	}

	/// Validate an extension
	async fn run_validation(args:CliArgs) -> Result<()> {
		dev_log!("extensions", "Validating extension");

		let extension_path = args
			.extension
			.ok_or_else(|| anyhow::anyhow!("Extension path required for validation"))?;

		let path = PathBuf::from(extension_path);

		let result = Self::validate_extension(&path, false).await?;

		if result.is_valid {
			dev_log!("extensions", "Extension validation passed");

			Ok(())
		} else {
			dev_log!("extensions", "error: extension validation failed");

			Err(anyhow::anyhow!("Validation failed"))
		}
	}

	/// Validate an extension manifest
	pub async fn validate_extension(path:&PathBuf, detailed:bool) -> Result<ValidationResult> {
		dev_log!("extensions", "Validating extension at: {:?}", path);

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
							dev_log!("extensions", "Valid package.json found");
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
				dev_log!("extensions", "Validation error: {}", error);
			}
		}

		Ok(ValidationResult { is_valid, errors })
	}

	/// Build a WASM module
	pub async fn build_wasm_module(
		source:PathBuf,

		output:PathBuf,

		_opt_level:String,

		_target:Option<String>,
	) -> Result<BuildResult> {
		dev_log!("wasm", "Building WASM module from: {:?}", source);

		dev_log!("wasm", "Output: {:?}", output);

		// For now, return a placeholder result
		// In production, this would invoke rustc/cargo with wasm32-wasi target
		Ok(BuildResult { success:true, output_path:output, compile_time_ms:0 })
	}

	/// List loaded extensions
	pub async fn list_extensions(_detailed:bool) -> Result<Vec<ExtensionInfo>> {
		dev_log!("extensions", "Listing extensions");

		// For now, return empty list
		// In production, this would query the extension manager
		Ok(Vec::new())
	}

	/// Create transport based on arguments
	fn create_transport(args:&CliArgs) -> Result<Transport> {
		match args.transport.as_str() {
			"grpc" => {
				use crate::Transport::gRPCTransport::gRPCTransport;

				Ok(Transport::gRPC(
					gRPCTransport::New(&args.grpc_address).context("Failed to create gRPC transport")?,
				))
			},

			"ipc" => {
				use crate::Transport::IPCTransport::IPCTransport;

				Ok(Transport::IPC(IPCTransport::New().context("Failed to create IPC transport")?))
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
		dev_log!("lifecycle", "Grove is running. Press Ctrl+C to stop.");

		tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");

		dev_log!("lifecycle", "Shutdown signal received");
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
