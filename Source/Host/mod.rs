//! Host Module
//!
//! Provides the core extension hosting functionality for Grove.
//! Manages extension lifecycle, activation, and API bridging.
//!
//! # Architecture
//!
//! ```text
//! +++++++++++++++++++++++++++++++++++++++++++
//! +         Extension Host                 +
//! +++++++++++++++++++++++++++++++++++++++++++
//! +  ExtensionHost  →  Main host controller+
//! +  ExtensionMgr   →  Extension discovery  +
//! +  Activation     →  Event handling       +
//! +  Lifecycle      →  Lifecycle management +
//! +  APIBridge      →  VS Code API proxy    +
//! +++++++++++++++++++++++++++++++++++++++++++
//!           +                    +
//!           ▼                    ▼
//! ++++++++++++++++++++  ++++++++++++++++++++
//! +  WASM Runtime    +  +  Transport       +
//! +  (Executes)      +  +  (Communicates)  +
//! ++++++++++++++++++++  ++++++++++++++++++++
//! ```
//!
//! # Key Components
//!
//! - [`ExtensionHost`] - Main extension host controller
//! - [`ExtensionManager`] - Extension discovery and loading
//! - [`Activation`] - Extension activation event handling
//! - [`Lifecycle`] - Extension lifecycle management
//! - [`APIBridge`] - VS Code API implementation bridge

pub mod Activation;
pub mod APIBridge;
pub mod ExtensionHost;
pub mod ExtensionManager;
pub mod Lifecycle;

// Re-exports for convenience - use module prefix to avoid E0255 conflicts
pub use Activation::{ActivationEngine, ActivationEvent};
pub use Lifecycle::{LifecycleEvent, LifecycleManager};
// Note: ExtensionHost, ExtensionManager, APIBridge must be accessed via module prefix
use anyhow::Result;

/// Host configuration
#[derive(Debug, Clone)]
pub struct HostConfig {
	/// Maximum number of concurrent extensions
	pub max_extensions:usize,
	/// Enable lazy activation
	pub lazy_activation:bool,
	/// Enable hot reloading
	pub hot_reload:bool,
	/// Extension discovery paths
	pub discovery_paths:Vec<String>,
	/// Enable API logging
	pub api_logging:bool,
	/// Activation timeout in milliseconds
	pub activation_timeout_ms:u64,
}

impl Default for HostConfig {
	fn default() -> Self {
		Self {
			max_extensions:100,
			lazy_activation:true,
			hot_reload:false,
			discovery_paths:vec!["~/.vscode/extensions".to_string(), "~/.grove/extensions".to_string()],
			api_logging:false,
			activation_timeout_ms:30000,
		}
	}
}

impl HostConfig {
	/// Create a new host configuration
	pub fn new() -> Self { Self::default() }

	/// Set maximum number of extensions
	pub fn with_max_extensions(mut self, max:usize) -> Self {
		self.max_extensions = max;
		self
	}

	/// Enable or disable lazy activation
	pub fn with_lazy_activation(mut self, enabled:bool) -> Self {
		self.lazy_activation = enabled;
		self
	}

	/// Enable or disable hot reloading
	pub fn with_hot_reload(mut self, enabled:bool) -> Self {
		self.hot_reload = enabled;
		self
	}

	/// Set activation timeout
	pub fn with_activation_timeout(mut self, timeout_ms:u64) -> Self {
		self.activation_timeout_ms = timeout_ms;
		self
	}

	/// Add a discovery path
	pub fn add_discovery_path(mut self, path:String) -> Self {
		self.discovery_paths.push(path);
		self
	}
}

/// Extension activation result
#[derive(Debug, Clone)]
pub struct ActivationResult {
	/// Extension ID
	pub extension_id:String,
	/// Activation success
	pub success:bool,
	/// Activation time in milliseconds
	pub time_ms:u64,
	/// Error message if failed
	pub error:Option<String>,
	/// Contributed items
	pub contributes:Vec<String>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_host_config_default() {
		let config = HostConfig::default();
		assert_eq!(config.max_extensions, 100);
		assert_eq!(config.lazy_activation, true);
	}

	#[test]
	fn test_host_config_builder() {
		let config = HostConfig::default()
			.with_max_extensions(50)
			.with_lazy_activation(false)
			.with_activation_timeout(60000);

		assert_eq!(config.max_extensions, 50);
		assert_eq!(config.lazy_activation, false);
		assert_eq!(config.activation_timeout_ms, 60000);
	}

	#[test]
	fn test_activation_result() {
		let result = ActivationResult {
			extension_id:"test.ext".to_string(),
			success:true,
			time_ms:100,
			error:None,
			contributes:vec!["command.test".to_string()],
		};

		assert_eq!(result.extension_id, "test.ext");
		assert!(result.success);
		assert_eq!(result.contributes.len(), 1);
	}
}
