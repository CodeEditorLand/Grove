//! WebAssembly Runtime Module
//!
//! This module provides WebAssembly runtime support using WASMtime,
//! enabling Grove to execute VS Code extensions compiled to WebAssembly.
//!
//! # Architecture
//!
//! ```text
//! +++++++++++++++++++++++++++++++++++++++
//! +        WASM Runtime                 +
//! +++++++++++++++++++++++++++++++++++++++
//! +  wasmty::Engine                    +
//! +  wasmty::Store                     +
//! +  wasmty::Module                    +
//! +  wasmty::Instance                  +
//! +++++++++++++++++++++++++++++++++++++++
//!           +                    +
//!           ▼                    ▼
//! ++++++++++++++++++++  ++++++++++++++++++++
//! +  HostBridge      +  +  MemoryManager   +
//! +  (Communication) +  +  (Memory mgmt)   +
//! ++++++++++++++++++++  ++++++++++++++++++++
//!           +
//!           ▼
//! ++++++++++++++++++++
//! +  FunctionExport  +
//! +  (Host exports)  +
//! ++++++++++++++++++++
//! ```
//!
//! # Key Components
//!
//! - [`Runtime`] - WASMtime engine and store management
//! - [`ModuleLoader`] - WASM module compilation and instantiation
//! - [`MemoryManager`] - WASM memory allocation and management
//! - [`HostBridge`] - Host-WASM function communication bridge
//! - [`FunctionExport`] - Export host functions to WASM

pub mod FunctionExport;

pub mod HostBridge;

pub mod MemoryManager;

pub mod ModuleLoader;

pub mod Runtime;

// Note: ModuleLoader, MemoryManager, HostBridge, FunctionExport must be
// accessed via module prefix
use anyhow::Result;

/// Default configuration for WASM runtime
/// Default memory limit in megabytes
pub const DEFAULT_MEMORY_LIMIT_MB:u64 = 512;

/// Default maximum execution time in milliseconds
pub const DEFAULT_MAX_EXECUTION_TIME_MS:u64 = 30000;

/// Default table size
pub const DEFAULT_TABLE_SIZE:u32 = 1024;

/// WASM runtime statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WASMStats {
	/// Number of loaded modules
	pub modules_loaded:usize,

	/// Number of active instances
	pub active_instances:usize,

	/// Total memory used ( MB)
	pub total_memory_mb:u64,

	/// Total execution time (ms)
	pub total_execution_time_ms:u64,

	/// Number of function calls
	pub function_calls:u64,
}

impl Default for WASMStats {
	fn default() -> Self {
		Self {
			modules_loaded:0,

			active_instances:0,

			total_memory_mb:0,

			total_execution_time_ms:0,

			function_calls:0,
		}
	}
}

/// Initialize WASM runtime with default configuration
///
/// # Example
///
/// ```rust,no_run
/// use grove::WASM;
///
/// # async fn example() -> anyhow::Result<()> {
/// let runtime = WASM::init_wasm_runtime().await?;
/// # Ok(())
/// # }
/// ```
pub async fn init_wasm_runtime() -> Result<Runtime::WASMRuntime> {
	Runtime::WASMRuntime::new(Runtime::WASMConfig::default()).await
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_default_config() {
		let config = Runtime::WASMConfig::default();

		assert_eq!(config.memory_limit_mb, DEFAULT_MEMORY_LIMIT_MB);
	}

	#[test]
	fn test_stats_default() {
		let stats = WASMStats::default();

		assert_eq!(stats.modules_loaded, 0);

		assert_eq!(stats.active_instances, 0);
	}
}
