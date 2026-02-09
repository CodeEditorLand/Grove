//! Extension Host Module
//!
//! Main extension host controller for Grove.
//! Manages the overall host lifecycle and coordinates extension execution.

use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, instrument, warn};

use crate::{
	Host::{Activation, ExtensionManager, HostConfig},
	Transport::Transport,
	WASM::Runtime::{WASMConfig, WASMRuntime},
};

/// Main extension host controller
pub struct ExtensionHost {
	/// Host configuration
	config:HostConfig,
	/// Transport for communication
	transport:Transport,
	/// Extension manager
	extension_manager:Arc<ExtensionManager>,
	/// Activation engine
	activation_engine:Arc<Activation::ActivationEngine>,
	/// WASM runtime
	wasm_runtime:Arc<WASMRuntime>,
	/// Active extensions
	active_extensions:Arc<RwLock<Vec<String>>>,
	/// Host state
	state:Arc<RwLock<HostState>>,
}

/// Host state
#[derive(Debug, Clone, PartialEq)]
pub enum HostState {
	/// Host has been created but not initialized
	Created,
	/// Host is ready to accept extensions
	Ready,
	/// Host is running with active extensions
	Running,
	/// Host is shutting down
	ShuttingDown,
	/// Host has been terminated
	Terminated,
}

/// Host statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostStats {
	/// Number of loaded extensions
	pub loaded_extensions:usize,
	/// Number of active extensions
	pub active_extensions:usize,
	/// Total number of activations
	pub total_activations:u64,
	/// Total activation time in milliseconds
	pub total_activation_time_ms:u64,
	/// Number of API calls made
	pub api_calls:u64,
	/// Number of errors encountered
	pub errors:u64,
	/// Host uptime in seconds
	pub uptime_seconds:u64,
}

impl ExtensionHost {
	/// Create a new extension host
	///
	/// # Arguments
	///
	/// * `transport` - The communication transport to use
	///
	/// # Example
	///
	/// ```rust,no_run
	/// use grove::{ExtensionHost, Transport};
	///
	/// # async fn example() -> anyhow::Result<()> {
	/// let transport = Transport::default();
	/// let host = ExtensionHost::new(transport).await?;
	/// # Ok(())
	/// # }
	/// ```
	#[instrument(skip(transport))]
	pub async fn new(transport:Transport) -> Result<Self> { Self::with_config(transport, HostConfig::default()).await }

	/// Create a new extension host with custom configuration
	#[instrument(skip(transport, config))]
	pub async fn with_config(transport:Transport, config:HostConfig) -> Result<Self> {
		info!("Creating extension host with config: {:?}", config);

		// Connect transport
		transport.connect().await.context("Failed to connect transport")?;

		// Create WASM runtime
		let wasm_config = WASMConfig::new(512, 30000, true);
		let wasm_runtime = Arc::new(WASMRuntime::new(wasm_config).await?);

		// Create extension manager
		let extension_manager = Arc::new(ExtensionManager::new(Arc::clone(&wasm_runtime), config.clone()));

		// Create activation engine
		let activation_engine = Arc::new(Activation::ActivationEngine::new(
			Arc::clone(&extension_manager),
			config.clone(),
		));

		info!("Extension host created successfully");

		Ok(Self {
			config,
			transport,
			extension_manager,
			activation_engine,
			wasm_runtime,
			active_extensions:Arc::new(RwLock::new(Vec::new())),
			state:Arc::new(RwLock::new(HostState::Created)),
		})
	}

	/// Load an extension from a path
	#[instrument(skip(self, path))]
	pub async fn load_extension(&self, path:&PathBuf) -> Result<String> {
		info!("Loading extension from: {:?}", path);

		let extension_id = self
			.extension_manager
			.load_extension(path)
			.await
			.context("Failed to load extension")?;

		info!("Extension loaded: {}", extension_id);

		*self.state.write().await = HostState::Ready;

		Ok(extension_id)
	}

	/// Unload an extension
	#[instrument(skip(self, extension_id))]
	pub async fn unload_extension(&self, extension_id:&str) -> Result<()> {
		info!("Unloading extension: {}", extension_id);

		self.extension_manager
			.unload_extension(extension_id)
			.await
			.context("Failed to unload extension")?;

		info!("Extension unloaded: {}", extension_id);

		Ok(())
	}

	/// Activate an extension
	#[instrument(skip(self, extension_id))]
	pub async fn activate(&self, extension_id:&str) -> Result<()> {
		info!("Activating extension: {}", extension_id);

		let start = std::time::Instant::now();

		let result = self
			.activation_engine
			.activate(extension_id)
			.await
			.context("Failed to activate extension")?;

		let elapsed = start.elapsed().as_millis() as u64;

		if result.success {
			info!("Extension activated in {}ms: {}", elapsed, extension_id);

			// Track active extension
			let mut active = self.active_extensions.write().await;
			if !active.contains(&extension_id.to_string()) {
				active.push(extension_id.to_string());
			}

			*self.state.write().await = HostState::Running;
		} else {
			error!("Extension activation failed: {}", extension_id);
		}

		Ok(())
	}

	/// Deactivate an extension
	#[instrument(skip(self, extension_id))]
	pub async fn deactivate(&self, extension_id:&str) -> Result<()> {
		info!("Deactivating extension: {}", extension_id);

		self.activation_engine
			.deactivate(extension_id)
			.await
			.context("Failed to deactivate extension")?;

		// Remove from active extensions
		let mut active = self.active_extensions.write().await;
		active.retain(|id| id != extension_id);

		info!("Extension deactivated: {}", extension_id);

		Ok(())
	}

	/// Activate all loaded extensions
	#[instrument(skip(self))]
	pub async fn activate_all(&self) -> Result<Vec<String>> {
		info!("Activating all extensions");

		let extensions = self.extension_manager.list_extensions().await;
		let mut activated = Vec::new();
		let mut failed = Vec::new();

		for extension_id in extensions {
			match self.activate(&extension_id).await {
				Ok(_) => activated.push(extension_id),
				Err(e) => {
					error!("Failed to activate {}: {}", extension_id, e);
					failed.push(extension_id);
				},
			}
		}

		warn!("Activated {} extensions, {} failed", activated.len(), failed.len());

		Ok(activated)
	}

	/// Deactivate all active extensions
	#[instrument(skip(self))]
	pub async fn deactivate_all(&self) -> Result<()> {
		info!("Deactivating all extensions");

		let active = self.active_extensions.read().await.clone();

		for extension_id in active {
			if let Err(e) = self.deactivate(&extension_id).await {
				error!("Failed to deactivate {}: {}", extension_id, e);
			}
		}

		*self.state.write().await = HostState::Ready;

		Ok(())
	}

	/// Get host statistics
	pub async fn stats(&self) -> HostStats {
		let active_extensions = self.active_extensions.read().await.len();
		let loaded_extensions = self.extension_manager.list_extensions().await.len();
		let extension_stats = self.extension_manager.stats().await;

		HostStats {
			loaded_extensions,
			active_extensions,
			total_activations:extension_stats.total_activations,
			total_activation_time_ms:extension_stats.total_activation_time_ms,
			api_calls:0, // Track through API bridge
			errors:extension_stats.errors,
			uptime_seconds:0, // Track from host start time
		}
	}

	/// Get host state
	pub async fn state(&self) -> HostState { *self.state.read().await }

	/// Get the transport
	pub fn transport(&self) -> &Transport { &self.transport }

	/// Get the extension manager
	pub fn extension_manager(&self) -> &Arc<ExtensionManager> { &self.extension_manager }

	/// Get the activation engine
	pub fn activation_engine(&self) -> &Arc<Activation::ActivationEngine> { &self.activation_engine }

	/// Get the WASM runtime
	pub fn wasm_runtime(&self) -> &Arc<WASMRuntime> { &self.wasm_runtime }

	/// Shutdown the host and clean up resources
	#[instrument(skip(self))]
	pub async fn shutdown(&self) -> Result<()> {
		info!("Shutting down extension host");

		*self.state.write().await = HostState::ShuttingDown;

		// Deactivate all extensions
		if let Err(e) = self.deactivate_all().await {
			error!("Error deactivating extensions during shutdown: {}", e);
		}

		// Close transport
		if let Err(e) = self.transport.close().await {
			error!("Error closing transport during shutdown: {}", e);
		}

		// Shutdown WASM runtime
		if let Err(e) = self.wasm_runtime.shutdown().await {
			error!("Error shutting down WASM runtime: {}", e);
		}

		*self.state.write().await = HostState::Terminated;

		info!("Extension host shutdown complete");

		Ok(())
	}
}

impl Drop for ExtensionHost {
	fn drop(&mut self) {
		info!("ExtensionHost dropped");
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_host_state() {
		assert_eq!(HostState::Created, HostState::Created);
		assert_eq!(HostState::Ready, HostState::Ready);
		assert_eq!(HostState::Running, HostState::Running);
	}

	#[test]
	fn test_host_stats_default() {
		let stats = HostStats::default();
		assert_eq!(stats.loaded_extensions, 0);
		assert_eq!(stats.active_extensions, 0);
	}

	#[test]
	fn test_host_config_default() {
		let config = HostConfig::default();
		assert_eq!(config.max_extensions, 100);
		assert_eq!(config.lazy_activation, true);
	}
}
