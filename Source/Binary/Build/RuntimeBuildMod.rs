//! Runtime Build Module
//!
//! Provides runtime construction for the Grove extension host.
//! Handles building and initializing the host runtime.

use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::{debug, info, instrument, warn};

use crate::{
	Host::{ExtensionHost::ExtensionHostImpl, HostConfig},
	Transport::Transport,
	WASM::Runtime::{WASMConfig, WASMRuntime},
};

/// Runtime build utilities
pub struct RuntimeBuild;

impl RuntimeBuild {
	/// Build a Groove extension host with the specified configuration
	#[instrument(skip(transport, wasm_runtime))]
	pub async fn build_host(
		transport:Transport,
		wasm_runtime:Arc<WASMRuntime>,
		host_config:HostConfig,
	) -> Result<ExtensionHostImpl> {
		info!("Building Grove extension host");

		// In a real implementation, we would use the provided wasm_runtime
		// For now, we create the host with default configuration

		let host = ExtensionHostImpl::with_config(transport, host_config.clone())
			.await
			.context("Failed to build extension host")?;

		info!("Extension host built successfully");

		Ok(host)
	}

	/// Build a Grove extension host with default WASM configuration
	pub async fn build_host_with_defaults(
		transport:Transport,
		wasi:bool,
		memory_limit_mb:u64,
		max_execution_time_ms:u64,
	) -> Result<ExtensionHostImpl> {
		info!("Building Grove extension host with defaults");

		let wasm_config = WASMConfig::new(memory_limit_mb, max_execution_time_ms, wasi);
		let wasm_runtime = Arc::new(WASMRuntime::new(wasm_config).await?);

		let host_config = HostConfig::default().with_activation_timeout(max_execution_time_ms);

		Self::build_host(transport, wasm_runtime, host_config).await
	}

	/// Build a minimal extension host for testing
	#[instrument(skip(transport))]
	pub async fn build_minimal_host(transport:Transport) -> Result<ExtensionHostImpl> {
		debug!("Building minimal extension host");

		let host_config = HostConfig::default().with_max_extensions(10).with_lazy_activation(true);

		let wasm_config = WASMConfig::new(64, 10000, false);
		let wasm_runtime = Arc::new(WASMRuntime::new(wasm_config).await?);

		Self::build_host(transport, wasm_runtime, host_config).await
	}

	/// Validate build configuration
	pub fn validate_config(config:&HostConfig) -> Result<()> {
		if config.max_extensions == 0 {
			return Err(anyhow::anyhow!("max_extensions must be at least 1"));
		}

		if config.activation_timeout_ms == 0 {
			return Err(anyhow::anyhow!("activation_timeout_ms must be at least 1"));
		}

		Ok(())
	}
}

impl Default for RuntimeBuild {
	fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_runtime_build_default() {
		let builder = RuntimeBuild::default();
		// Just test that it can be created
		let _ = builder;
	}

	#[test]
	fn test_validate_config() {
		let valid_config = HostConfig::default();
		assert!(RuntimeBuild::validate_config(&valid_config).is_ok());

		let mut invalid_config = HostConfig::default();
		invalid_config.max_extensions = 0;
		assert!(RuntimeBuild::validate_config(&invalid_config).is_err());
	}
}
