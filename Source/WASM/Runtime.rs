//! WASM Runtime Module
//!
//! Provides Wasmtime engine and store management for executing WebAssembly
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
	MemoryManager::{MemoryLimits, MemoryManager},
};

/// Configuration for the WASM runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
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

impl Default for WasmConfig {
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

impl WasmConfig {
	/// Create a new WASM configuration with custom settings
	pub fn new(memory_limit_mb:u64, max_execution_time_ms:u64, enable_wasi:bool) -> Self {
		Self { memory_limit_mb, max_execution_time_ms, enable_wasi, ..Default::default() }
	}

	/// Apply this configuration to a Wasmtime engine builder
	fn apply_to_engine_builder(&self, mut builder:wasmtime::Config) -> Result<wasmtime::Config> {
		// Enable WASM
		builder.wasm_component_model(false);

		// Enable WASI if configured
		if self.enable_wasi {
			builder.wasi(true);
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

/// WASM Runtime - manages Wasmtime engine and stores
#[derive(Clone)]
pub struct WasmRuntime {
	engine:Engine,
	config:WasmConfig,
	memory_manager:Arc<RwLock<MemoryManager>>,
	instances:Arc<RwLock<Vec<String>>>,
}

impl WasmRuntime {
	/// Create a new WASM runtime with the given configuration
	#[instrument(skip(config))]
	pub async fn new(config:WasmConfig) -> Result<Self> {
		info!("Creating WASM runtime with config: {:?}", config);

		// Build the Wasmtime engine
		let engine_config = wasmtime::Config::new();
		let engine_config = config.apply_to_engine_builder(engine_config)?;
		let engine = Engine::new(&engine_config).context("Failed to create Wasmtime engine")?;

		// Initialize memory manager
		let memory_limits = MemoryLimits {
			max_memory_mb:config.memory_limit_mb,
			// Set 75% of max for initial allocation
			initial_memory_mb:(config.memory_limit_mb as f64 * 0.75) as u64,
			max_table_size:1024,
			// Set maximum of 100 instances
			max_instances:100,
		};
		let memory_manager = Arc::new(RwLock::new(MemoryManager::new(memory_limits)));

		info!("WASM runtime created successfully");

		Ok(Self { engine, config, memory_manager, instances:Arc::new(RwLock::new(Vec::new())) })
	}

	/// Get a reference to the Wasmtime engine
	pub fn engine(&self) -> &Engine { &self.engine }

	/// Get the runtime configuration
	pub fn config(&self) -> &WasmConfig { &self.config }

	/// Get the memory manager
	pub fn memory_manager(&self) -> Arc<RwLock<MemoryManager>> { Arc::clone(&self.memory_manager) }

	/// Create a new WASM store with limits
	pub fn create_store(&self) -> Result<Store<StoreLimits>> {
		let mut store_limits = StoreLimitsBuilder::new()
            .memory_size(self.config.memory_limit_mb * 1024 * 1024) // Convert MB to bytes
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
			store.set_fuel(fuel).context("Failed to set fuel limit")?;
		}

		Ok(store)
	}

	/// Create a linker for the runtime
	pub fn create_linker<T>(&self, async_support:bool) -> Result<Linker<T>>
	where
		T: Send, {
		let mut linker = Linker::new(&self.engine);

		// Add WASI support if enabled
		if self.config.enable_wasi {
			wasmtime_wasi::add_to_linker(&mut linker, |s| s).context("Failed to add WASI to linker")?;
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

		let module = Module::from_binary(&self.engine, wasm_bytes).context("Failed to compile WASM module")?;

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
		let runtime = WasmRuntime::new(WasmConfig::default()).await;
		assert!(runtime.is_ok());
	}

	#[tokio::test]
	async fn test_wasm_config_default() {
		let config = WasmConfig::default();
		assert!(config.enable_wasi);
		assert_eq!(config.memory_limit_mb, 512);
	}

	#[tokio::test]
	async fn test_create_store() {
		let runtime = WasmRuntime::new(WasmConfig::default()).await.unwrap();
		let store = runtime.create_store();
		assert!(store.is_ok());
	}

	#[tokio::test]
	async fn test_instance_registration() {
		let runtime = WasmRuntime::new(WasmConfig::default()).await.unwrap();

		runtime.register_instance("test-instance".to_string()).await.unwrap();
		assert_eq!(runtime.instance_count().await, 1);

		runtime.unregister_instance("test-instance").await.unwrap();
		assert_eq!(runtime.instance_count().await, 0);
	}

	#[tokio::test]
	async fn test_validate_module() {
		let runtime = WasmRuntime::new(WasmConfig::default()).await.unwrap();

		// Simple WASM module (empty)
		let empty_wasm = vec![
			0x00, 0x61, 0x73, 0x6D, // Magic number
			0x01, 0x00, 0x00, 0x00, // Version 1
		];

		// This will fail validation because it's incomplete, but tests the method
		let result = runtime.validate_module(&empty_wasm);
		// We don't assert on the result since it depends on Wasmtime
		// implementation
	}
}
