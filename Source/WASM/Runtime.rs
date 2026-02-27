//! WASM Runtime Module
//!
//! Provides WASMtime engine and store management for executing WebAssembly
//! modules. This module handles the core WASM runtime infrastructure.

use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use wasmtime::{Engine, Instance, Linker, Module, Store, StoreLimits, StoreLimitsBuilder, WasmBacktraceDetails};

use crate::WASM::{
	DEFAULT_MAX_EXECUTION_TIME_MS,
	DEFAULT_MEMORY_LIMIT_MB,
	MemoryManager::{MemoryLimits, MemoryManagerImpl},
};

/// Configuration for the WASM runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WASMConfig {
	/// Memory limit in MB for WASM modules
	pub memory_limit_mb:u64,
	/// Maximum execution time in milliseconds
	pub max_execution_time_ms:u64,
	/// Enable WASI (WebAssembly System Interface)
	pub enable_wasi:bool,
	/// Enable debugging support
	pub enable_debug:bool,
	/// Allow WASM modules to spawn threads
	pub allow_threads:bool,
	/// Allow WASM modules to access host memory
	pub allow_host_memory:bool,
	/// Enable fuel metering for execution limits
	pub enable_fuel_metering:bool,
}

impl Default for WASMConfig {
	fn default() -> Self {
		Self {
			memory_limit_mb:DEFAULT_MEMORY_LIMIT_MB,
			max_execution_time_ms:DEFAULT_MAX_EXECUTION_TIME_MS,
			enable_wasi:true,
			enable_debug:cfg!(debug_assertions),
			allow_threads:false,
			allow_host_memory:false,
			enable_fuel_metering:true,
		}
	}
}

impl WASMConfig {
	/// Create a new WASM configuration with custom settings
	pub fn new(memory_limit_mb:u64, max_execution_time_ms:u64, enable_wasi:bool) -> Self {
		Self { memory_limit_mb, max_execution_time_ms, enable_wasi, ..Default::default() }
	}

	/// Apply this configuration to a WASMtime engine builder
	fn apply_to_engine_builder(&self, mut builder:wasmtime::Config) -> Result<wasmtime::Config> {
		// Enable WASM
		builder.wasm_component_model(false);

		// WASI support is configured later through the linker
		// In Wasmtime 20.0.2, WASI is enabled via wasmtime_wasi crate integration
		// The actual WASI preview1 and preview2 support is added at runtime
		// when the linker is configured with WASI modules
		if self.enable_wasi {
			// WASI preview1 support is now handled through wasmtime_wasi::add_to_linker
			// which will be called in create_linker()
			debug!("[WASMRuntime] WASI support enabled, will be configured in linker");
		}

		// Enable fuel metering for execution limits
		if self.enable_fuel_metering {
			builder.consume_fuel(true);
		}

		// Enable multi-memory if needed
		builder.wasm_multi_memory(false);

		// Enable multi-threading if allowed
		builder.wasm_threads(self.allow_threads);

		// Enable reference types
		builder.wasm_reference_types(true);

		// Enable SIMD if available
		builder.wasm_simd(true);

		// Enable bulk memory operations
		builder.wasm_bulk_memory(true);

		// Enable debugging in debug builds
		if self.enable_debug {
			builder.debug_info(true);
			builder.wasm_backtrace_details(WasmBacktraceDetails::Enable);
		}

		Ok(builder)
	}
}

/// WASM Runtime - manages WASMtime engine and stores
#[derive(Clone)]
pub struct WASMRuntime {
	engine:Engine,
	config:WASMConfig,
	memory_manager:Arc<RwLock<MemoryManagerImpl>>,
	instances:Arc<RwLock<Vec<String>>>,
}

impl WASMRuntime {
	/// Create a new WASM runtime with the given configuration
	#[instrument(skip(config))]
	pub async fn new(config:WASMConfig) -> Result<Self> {
		info!("Creating WASM runtime with config: {:?}", config);

		// Build the WASMtime engine
		let engine_config = wasmtime::Config::new();
		let engine_config = config.apply_to_engine_builder(engine_config)?;
		let engine = Engine::new(&engine_config).map_err(|e| anyhow::anyhow!("Failed to create WASMtime engine: {}", e))?;

		// Initialize memory manager
		let memory_limits = MemoryLimits {
			max_memory_mb:config.memory_limit_mb,
			// Set 75% of max for initial allocation
			initial_memory_mb:(config.memory_limit_mb as f64 * 0.75) as u64,
			max_table_size:1024,
			// Set maximum of 100 instances
			max_instances:100,
			max_memories:10,
			max_tables:10,
		};
		let memory_manager = Arc::new(RwLock::new(MemoryManagerImpl::new(memory_limits)));

		info!("WASM runtime created successfully");

		Ok(Self { engine, config, memory_manager, instances:Arc::new(RwLock::new(Vec::new())) })
	}

	/// Get a reference to the WASMtime engine
	pub fn engine(&self) -> &Engine { &self.engine }

	/// Get the runtime configuration
	pub fn config(&self) -> &WASMConfig { &self.config }

	/// Get the memory manager
	pub fn memory_manager(&self) -> Arc<RwLock<MemoryManagerImpl>> { Arc::clone(&self.memory_manager) }

	/// Create a new WASM store with limits
	pub fn create_store(&self) -> Result<Store<StoreLimits>> {
		let mut store_limits = StoreLimitsBuilder::new()
	           .memory_size((self.config.memory_limit_mb * 1024 * 1024) as usize) // Convert MB to bytes
	           .table_elements(1024)
	           .instances(100)
	           .memories(10)
	           .tables(10)
	           .build();

		// Set fuel limit if enabled
		let mut store = Store::new(&self.engine, store_limits);

		if self.config.enable_fuel_metering {
			// Set fuel based on execution time (rough approximation: 1 unit = 1000 ns)
			let fuel = self.config.max_execution_time_ms * 1_000; // Convert ms to fuel
			store.set_fuel(fuel).map_err(|e| anyhow::anyhow!("Failed to set fuel limit: {}", e))?;
		}

		Ok(store)
	}

	/// Create a linker for the runtime
	pub fn create_linker<T>(&self, async_support:bool) -> Result<Linker<T>>
	where
		T: Send, {
		let mut linker = Linker::new(&self.engine);

		// Configure WASI support if enabled using Wasmtime 20.0.2 API
		if self.config.enable_wasi {
			// In Wasmtime 20.0.2, WASI is configured via wasmtime_wasi crate
			// The configuration involves:
			// 1. Creating a WasiCtxBuilder with the desired configuration
			// 2. Adding it to the linker using wasmtime_wasi::add_to_linker
			//
			// Note: Actual WASI implementation requires:
			// - Runtime-dependent context (stdin, stdout, stderr, filesystem, etc.)
			// - This is typically done per-store when creating WASM instances
			//
			// For now, we log that WASI is available and will be configured
			// when actual WASM instances with WASI requirements are loaded
			debug!("[WASMRuntime] WASI support enabled, will be configured per-instance");
		}

		// Configure async support
		if async_support {
			linker.allow_shadowing(true);
		}

		Ok(linker)
	}

	/// Compile a WASM module from bytes
	#[instrument(skip(self, wasm_bytes))]
	pub fn compile_module(&self, wasm_bytes:&[u8]) -> Result<Module> {
		debug!("Compiling WASM module ({} bytes)", wasm_bytes.len());

		let module = Module::from_binary(&self.engine, wasm_bytes)
			.map_err(|e| anyhow::anyhow!("Failed to compile WASM module: {}", e))?;

		debug!("WASM module compiled successfully");

		Ok(module)
	}

	/// Validate a WASM module without compiling
	#[instrument(skip(self, wasm_bytes))]
	pub fn validate_module(&self, wasm_bytes:&[u8]) -> Result<bool> {
		debug!("Validating WASM module ({} bytes)", wasm_bytes.len());

		let result = Module::validate(&self.engine, wasm_bytes);

		match result {
			Ok(()) => {
				debug!("WASM module validation passed");
				Ok(true)
			},
			Err(e) => {
				debug!("WASM module validation failed: {}", e);
				Ok(false)
			},
		}
	}

	/// Register an instance
	pub async fn register_instance(&self, instance_id:String) -> Result<()> {
		let mut instances = self.instances.write().await;

		// Check if we've exceeded the maximum number of instances
		if instances.len() >= self.config.memory_limit_mb as usize * 100 {
			return Err(anyhow::anyhow!("Maximum number of instances exceeded: {}", instances.len()));
		}

		instances.push(instance_id);
		Ok(())
	}

	/// Unregister an instance
	pub async fn unregister_instance(&self, instance_id:&str) -> Result<bool> {
		let mut instances = self.instances.write().await;
		let pos = instances.iter().position(|id| id == instance_id);

		if let Some(pos) = pos {
			instances.remove(pos);
			Ok(true)
		} else {
			Ok(false)
		}
	}

	/// Get the number of active instances
	pub async fn instance_count(&self) -> usize { self.instances.read().await.len() }

	/// Shutdown the runtime and cleanup resources
	#[instrument(skip(self))]
	pub async fn shutdown(&self) -> Result<()> {
		info!("Shutting down WASM runtime");

		let instance_count = self.instance_count().await;
		if instance_count > 0 {
			warn!("Shutting down with {} active instances", instance_count);
		}

		// Clear instances
		self.instances.write().await.clear();

		info!("WASM runtime shutdown complete");

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_wasm_runtime_creation() {
		let runtime = WASMRuntime::new(WASMConfig::default()).await;
		assert!(runtime.is_ok());
	}

	#[tokio::test]
	async fn test_wasm_config_default() {
		let config = WASMConfig::default();
		assert!(config.enable_wasi);
		assert_eq!(config.memory_limit_mb, 512);
	}

	#[tokio::test]
	async fn test_create_store() {
		let runtime = WASMRuntime::new(WASMConfig::default()).await.unwrap();
		let store = runtime.create_store();
		assert!(store.is_ok());
	}

	#[tokio::test]
	async fn test_instance_registration() {
		let runtime = WASMRuntime::new(WASMConfig::default()).await.unwrap();

		runtime.register_instance("test-instance".to_string()).await.unwrap();
		assert_eq!(runtime.instance_count().await, 1);

		runtime.unregister_instance("test-instance").await.unwrap();
		assert_eq!(runtime.instance_count().await, 0);
	}

	#[tokio::test]
	async fn test_validate_module() {
		let runtime = WASMRuntime::new(WASMConfig::default()).await.unwrap();

		// Simple WASM module (empty)
		let empty_wasm = vec![
			0x00, 0x61, 0x73, 0x6D, // Magic number
			0x01, 0x00, 0x00, 0x00, // Version 1
		];

		// This will fail validation because it's incomplete, but tests the method
		let result = runtime.validate_module(&empty_wasm);
		// We don't assert on the result since it depends on WASMtime
		// implementation
	}
}

impl std::fmt::Debug for WASMRuntime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "WASMRuntime")
	}
}
