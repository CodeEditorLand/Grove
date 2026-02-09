//! WASM Transport Implementation
//!
//! Provides direct communication with WASM modules.
//! Handles calls to and from WebAssembly instances.

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::{
	Transport::{Strategy::TransportStats, TransportConfig},
	WASM::{
		HostBridge,
		MemoryManager::{MemoryLimits, MemoryManager},
		Runtime::{WasmConfig, WasmRuntime},
		WasmStats,
	},
};

/// WASM transport for direct module communication
#[derive(Clone)]
pub struct WasmTransport {
	/// WASM runtime
	runtime:Arc<WasmRuntime>,
	/// Memory manager
	memory_manager:Arc<RwLock<MemoryManager>>,
	/// Host bridge for communication
	bridge:Arc<HostBridge>,
	/// Loaded modules
	modules:Arc<RwLock<HashMap<String, WasmModuleInfo>>>,
	/// Transport configuration
	config:TransportConfig,
	/// Connection state
	connected:Arc<RwLock<bool>>,
	/// Transport statistics
	stats:Arc<RwLock<TransportStats>>,
}

/// Information about a loaded WASM module
#[derive(Debug, Clone)]
pub struct WasmModuleInfo {
	/// Module ID
	pub id:String,
	/// Module name (if available)
	pub name:Option<String>,
	/// Path to module file
	pub path:Option<PathBuf>,
	/// Module loaded timestamp
	pub loaded_at:u64,
	/// Function statistics
	pub function_stats:HashMap<String, FunctionCallStats>,
}

/// Statistics for function calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallStats {
	/// Number of calls
	pub call_count:u64,
	/// Total execution time in microseconds
	pub total_time_us:u64,
	/// Last call timestamp
	pub last_call_at:Option<u64>,
	/// Number of errors
	pub error_count:u64,
}

impl FunctionCallStats {
	/// Record a successful function call
	pub fn record_call(&mut self, time_us:u64) {
		self.call_count += 1;
		self.total_time_us += time_us;
		self.last_call_at = Some(
			std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_secs())
				.unwrap_or(0),
		);
	}

	/// Record a failed function call
	pub fn record_error(&mut self) { self.error_count += 1; }
}

impl Default for FunctionCallStats {
	fn default() -> Self { Self { call_count:0, total_time_us:0, last_call_at:None, error_count:0 } }
}

impl WasmTransport {
	/// Create a new WASM transport with default configuration
	pub fn new(enable_wasi:bool, memory_limit_mb:u64, max_execution_time_ms:u64) -> anyhow::Result<Self> {
		let config = WasmConfig::new(memory_limit_mb, max_execution_time_ms, enable_wasi);

		// Create runtime - this would normally be async, but for now we do it
		// synchronously In production, this would need to be properly awaited
		let runtime = Arc::new(
			tokio::runtime::Runtime::new()
				.map_err(|e| anyhow::anyhow!("Failed to create tokio runtime: {}", e))?
				.block_on(WasmRuntime::new(config.clone())),
		);

		let memory_limits = MemoryLimits::new(memory_limit_mb, (memory_limit_mb as f64 * 0.75) as u64, 100);
		let memory_manager = Arc::new(RwLock::new(MemoryManager::new(memory_limits)));
		let bridge = Arc::new(HostBridge::new());

		Ok(Self {
			runtime,
			memory_manager,
			bridge,
			modules:Arc::new(RwLock::new(HashMap::new())),
			config:TransportConfig::default(),
			connected:Arc::new(RwLock::new(true)), // WASM transport is always "connected" locally
			stats:Arc::new(RwLock::new(TransportStats::default())),
		})
	}

	/// Create a new WASM transport with custom configuration
	pub fn with_config(wasm_config:WasmConfig, transport_config:TransportConfig) -> anyhow::Result<Self> {
		let runtime = Arc::new(
			tokio::runtime::Runtime::new()
				.map_err(|e| anyhow::anyhow!("Failed to create tokio runtime: {}", e))?
				.block_on(WasmRuntime::new(wasm_config.clone())),
		);

		let memory_limits = MemoryLimits::new(
			wasm_config.memory_limit_mb,
			(wasm_config.memory_limit_mb as f64 * 0.75) as u64,
			100,
		);
		let memory_manager = Arc::new(RwLock::new(MemoryManager::new(memory_limits)));
		let bridge = Arc::new(HostBridge::new());

		Ok(Self {
			runtime,
			memory_manager,
			bridge,
			modules:Arc::new(RwLock::new(HashMap::new())),
			config:transport_config,
			connected:Arc::new(RwLock::new(true)),
			stats:Arc::new(RwLock::new(TransportStats::default())),
		})
	}

	/// Get a reference to the WASM runtime
	pub fn runtime(&self) -> &Arc<WasmRuntime> { &self.runtime }

	/// Get a reference to the memory manager
	pub fn memory_manager(&self) -> &Arc<RwLock<MemoryManager>> { &self.memory_manager }

	/// Get a reference to the host bridge
	pub fn bridge(&self) -> &Arc<HostBridge> { &self.bridge }

	/// Get all loaded modules
	pub async fn get_modules(&self) -> HashMap<String, WasmModuleInfo> { self.modules.read().await.clone() }

	/// Get WASM runtime statistics
	pub async fn get_wasm_stats(&self) -> WasmStats {
		let memory_manager = self.memory_manager.read().await;
		let managers = self.modules.read().await;

		WasmStats {
			modules_loaded:managers.len(),
			active_instances:managers.len(), // In real implementation, track instances
			total_memory_mb:memory_manager.current_usage_mb() as u64,
			total_execution_time_ms:0, // Track from actual calls
			function_calls:self.stats.read().await.messages_sent,
		}
	}

	/// Call a function in a WASM module
	#[instrument(skip(self, module_id, function_name, args))]
	pub async fn call_wasm_function(
		&self,
		module_id:&str,
		function_name:&str,
		args:Vec<Bytes>,
	) -> anyhow::Result<Bytes> {
		let start = std::time::Instant::now();

		debug!(
			"Calling WASM function: {}::{} with {} arguments",
			module_id,
			function_name,
			args.len()
		);

		let modules = self.modules.read().await;
		let module = modules
			.get(module_id)
			.ok_or_else(|| anyhow::anyhow!("Module not found: {}", module_id))?;

		// In a real implementation, this would call the actual WASM function
		// For now, we return a mock response
		let response = Bytes::new();

		// Update statistics
		let mut modules_mut = self.modules.write().await;
		if let Some(module) = modules_mut.get_mut(module_id) {
			let stats = module.function_stats.entry(function_name.to_string()).or_default();
			stats.record_call(start.elapsed().as_micros() as u64);
		}

		drop(modules_mut);

		// Update transport statistics
		let mut stats = self.stats.write().await;
		stats.record_sent(args.iter().map(|b| b.len() as u64).sum(), start.elapsed().as_micros() as u64);
		stats.record_received(response.len() as u64);

		Ok(response)
	}
}

#[async_trait]
impl super::super::Strategy::TransportStrategy for WasmTransport {
	type Error = WasmTransportError;

	#[instrument(skip(self))]
	async fn connect(&self) -> Result<(), Self::Error> {
		info!("WASM transport connecting");

		// WASM transport is always "connected" locally
		*self.connected.write().await = true;

		info!("WASM transport connected");

		Ok(())
	}

	#[instrument(skip(self, request))]
	async fn send(&self, request:&[u8]) -> Result<Vec<u8>, Self::Error> {
		let start = std::time::Instant::now();

		if !self.is_connected() {
			return Err(WasmTransportError::NotConnected);
		}

		debug!("Sending WASM transport request ({} bytes)", request.len());

		// Parse request - it should contain module ID and function name
		// For simplicity, we use a minimal format: module_id:function_name:base64_args
		let request_str =
			std::str::from_utf8(request).map_err(|e| WasmTransportError::InvalidRequest(e.to_string()))?;

		let parts:Vec<&str> = request_str.splitn(3, ':').collect();
		if parts.len() < 3 {
			return Err(WasmTransportError::InvalidRequest("Invalid request format".to_string()));
		}

		let module_id = parts[0];
		let function_name = parts[1];
		let args_base64 = parts[2];

		// Decode arguments from base64
		let args = vec![Bytes::from(
			base64::decode(args_base64).map_err(|e| WasmTransportError::InvalidRequest(e.to_string()))?,
		)];

		// Call the WASM function
		let response = self
			.call_wasm_function(module_id, function_name, args)
			.await
			.map_err(|e| WasmTransportError::FunctionCallFailed(e.to_string()))?;

		// Convert response to Vec<u8>
		let response_vec = response.to_vec();

		let latency_us = start.elapsed().as_micros() as u64;

		debug!("WASM transport request completed in {}µs", latency_us);

		Ok(response_vec)
	}

	#[instrument(skip(self, data))]
	async fn send_no_response(&self, data:&[u8]) -> Result<(), Self::Error> {
		if !self.is_connected() {
			return Err(WasmTransportError::NotConnected);
		}

		debug!("Sending WASM transport request without response ({} bytes)", data.len());

		// For fire-and-forget calls, we still execute but ignore the response
		self.send(data).await?;
		Ok(())
	}

	#[instrument(skip(self))]
	async fn close(&self) -> Result<(), Self::Error> {
		info!("Closing WASM transport");

		*self.connected.write().await = false;

		info!("WASM transport closed");

		Ok(())
	}

	fn is_connected(&self) -> bool { self.connected.blocking_read().to_owned() }
}

/// WASM transport errors
#[derive(Debug, thiserror::Error)]
pub enum WasmTransportError {
	#[error("Module not found: {0}")]
	ModuleNotFound(String),

	#[error("Function not found: {0}")]
	FunctionNotFound(String),

	#[error("Function call failed: {0}")]
	FunctionCallFailed(String),

	#[error("Memory error: {0}")]
	MemoryError(String),

	#[error("Runtime error: {0}")]
	RuntimeError(String),

	#[error("Invalid request: {0}")]
	InvalidRequest(String),

	#[error("Not connected")]
	NotConnected,

	#[error("Compilation failed: {0}")]
	CompilationFailed(String),

	#[error("Timeout")]
	Timeout,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_wasm_transport_creation() {
		let result = WasmTransport::new(true, 512, 30000);
		assert!(result.is_ok());
		let transport = result.unwrap();
		// WASM transport should always be connected
		assert!(transport.is_connected());
	}

	#[test]
	fn test_function_call_stats() {
		let mut stats = FunctionCallStats::default();
		stats.record_call(100);
		assert_eq!(stats.call_count, 1);
		assert_eq!(stats.total_time_us, 100);
		assert!(stats.last_call_at.is_some());
	}

	#[tokio::test]
	async fn test_wasm_transport_not_connected_after_close() {
		let transport = WasmTransport::new(true, 512, 30000).unwrap();
		transport.close().await.unwrap();
		assert!(!transport.is_connected());
	}

	#[tokio::test]
	async fn test_get_wasm_stats() {
		let transport = WasmTransport::new(true, 512, 30000).unwrap();
		let stats = transport.get_wasm_stats().await;
		assert_eq!(stats.modules_loaded, 0);
		assert_eq!(stats.active_instances, 0);
	}
}
