//! Function Export Module
//!
//! Handles exporting host functions to WASM modules.
//! Provides registration and management of functions that WASM can call.

use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use wasmtime::{Caller, Extern, Func, Linker, Store};

use crate::WASM::HostBridge::{FunctionSignature, HostBridge, HostFunctionCallback, ParamType, ReturnType};

/// Host function registry for WASM exports
pub struct HostFunctionRegistry {
	/// Registered host functions
	functions:Arc<RwLock<HashMap<String, RegisteredHostFunction>>>,
	/// Associated host bridge
	bridge:Arc<HostBridge>,
}

/// Registered host function with metadata
#[derive(Debug, Clone)]
struct RegisteredHostFunction {
	/// Function name
	name:String,
	/// Function signature
	signature:FunctionSignature,
	/// Synchronous callback
	callback:Option<HostFunctionCallback>,
	/// Registration timestamp
	registered_at:u64,
	/// Call statistics
	stats:FunctionStats,
}

/// Function statistics
#[derive(Debug, Clone, Default)]
pub struct FunctionStats {
	/// Number of times called
	pub call_count:u64,
	/// Total execution time in nanoseconds
	pub total_execution_ns:u64,
	/// Last call timestamp
	pub last_call_at:Option<u64>,
	/// Number of errors
	pub error_count:u64,
}

/// Export configuration for WASM functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
	/// Enable function export by default
	pub auto_export:bool,
	/// Enable timing statistics
	pub enable_stats:bool,
	/// Maximum number of functions that can be exported
	pub max_functions:usize,
	/// Function name prefix for exports
	pub name_prefix:Option<String>,
}

impl Default for ExportConfig {
	fn default() -> Self {
		Self {
			auto_export:true,
			enable_stats:true,
			max_functions:1000,
			name_prefix:Some("host_".to_string()),
		}
	}
}

/// Function export for WASM
pub struct FunctionExport {
	registry:Arc<HostFunctionRegistry>,
	config:ExportConfig,
}

impl FunctionExport {
	/// Create a new function export manager
	pub fn new(bridge:Arc<HostBridge>) -> Self {
		Self {
			registry:Arc::new(HostFunctionRegistry { functions:Arc::new(RwLock::new(HashMap::new())), bridge }),
			config:ExportConfig::default(),
		}
	}

	/// Create with custom configuration
	pub fn with_config(bridge:Arc<HostBridge>, config:ExportConfig) -> Self {
		Self {
			registry:Arc::new(HostFunctionRegistry { functions:Arc::new(RwLock::new(HashMap::new())), bridge }),
			config,
		}
	}

	/// Register a host function for export to WASM
	#[instrument(skip(self, callback))]
	pub async fn register_function(
		&self,
		name:&str,
		signature:FunctionSignature,
		callback:HostFunctionCallback,
	) -> Result<()> {
		info!("Registering host function for export: {}", name);

		let functions = self.registry.functions.read().await;

		// Check max function limit
		if functions.len() >= self.config.max_functions {
			return Err(anyhow::anyhow!(
				"Maximum number of exported functions reached: {}",
				self.config.max_functions
			));
		}

		drop(functions);

		let mut functions = self.registry.functions.write().await;

		let registered_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();

		functions.insert(
			name.to_string(),
			RegisteredHostFunction {
				name:name.to_string(),
				signature,
				callback:Some(callback),
				registered_at,
				stats:FunctionStats::default(),
			},
		);

		debug!("Host function registered for WASM export: {}", name);
		Ok(())
	}

	/// Register multiple host functions
	#[instrument(skip(self, callbacks))]
	pub async fn register_functions(
		&self,
		signatures:Vec<FunctionSignature>,
		callbacks:Vec<HostFunctionCallback>,
	) -> Result<()> {
		if signatures.len() != callbacks.len() {
			return Err(anyhow::anyhow!("Number of signatures must match number of callbacks"));
		}

		for (sig, callback) in signatures.into_iter().zip(callbacks) {
			let name = sig.name.clone();
			self.register_function(&name, sig, callback).await?;
		}

		Ok(())
	}

	/// Export all registered functions to a WASMtime linker
	#[instrument(skip(self, linker))]
	pub async fn export_to_linker<T>(&self, linker:&mut Linker<T>) -> Result<()>
	where
		T: Send + 'static, {
		info!(
			"Exporting {} host functions to linker",
			self.registry.functions.read().await.len()
		);

		let functions = self.registry.functions.read().await;

		for (name, func) in functions.iter() {
			self.export_single_function(linker, name, func)?;
		}

		info!("All host functions exported to linker");
		Ok(())
	}

	/// Export a single function to the linker
	fn export_single_function<T>(&self, linker:&mut Linker<T>, name:&str, func:&RegisteredHostFunction) -> Result<()>
	where
		T: Send + 'static, {
		debug!("Exporting function: {}", name);

		let callback = func
			.callback
			.ok_or_else(|| anyhow::anyhow!("No callback available for function: {}", name))?;

		let func_name = if let Some(prefix) = &self.config.name_prefix {
			format!("{}{}", prefix, name)
		} else {
			name.to_string()
		};

		// Create a wrapper function that handles stats and error handling
		let wrapped_callback =
			move |mut caller:Caller<'_, T>, args:&[wasmtime::Val]| -> Result<Vec<wasmtime::Val>, wasmtime::Trap> {
				let start = std::time::Instant::now();

				// Convert args to bytes
				let args_bytes:Result<Vec<bytes::Bytes>, _> = args
					.iter()
					.map(|arg| {
						match arg {
							wasmtime::Val::I32(i) => {
								serde_json::to_vec(i)
									.map(bytes::Bytes::from)
									.map_err(|e| anyhow::anyhow!("Serialization error: {}", e))
							},
							wasmtime::Val::I64(i) => {
								serde_json::to_vec(i)
									.map(bytes::Bytes::from)
									.map_err(|e| anyhow::anyhow!("Serialization error: {}", e))
							},
							wasmtime::Val::F32(f) => {
								serde_json::to_vec(f)
									.map(bytes::Bytes::from)
									.map_err(|e| anyhow::anyhow!("Serialization error: {}", e))
							},
							wasmtime::Val::F64(f) => {
								serde_json::to_vec(f)
									.map(bytes::Bytes::from)
									.map_err(|e| anyhow::anyhow!("Serialization error: {}", e))
							},
							_ => Err(anyhow::anyhow!("Unsupported argument type")),
						}
					})
					.collect();

				let args_bytes =
					args_bytes.map_err(|e| wasmtime::Trap::new(format!("Argument conversion failed: {}", e)))?;

				// Call the callback
				let result = callback(args_bytes);

				match result {
					Ok(response_bytes) => {
						// Deserialize response
						let result_val:serde_json::Value = serde_json::from_slice(&response_bytes)
							.map_err(|e| wasmtime::Trap::new(format!("Response deserialization failed: {}", e)))?;

						let ret_val = match result_val {
							serde_json::Value::Number(n) => {
								if let Some(i) = n.as_i64() {
									wasmtime::Val::I32(i as i32)
								} else if let Some(f) = n.as_f64() {
									wasmtime::Val::F64(f as f64)
								} else {
									return Err(wasmtime::Trap::new("Invalid return value type"));
								}
							},
							_ => return Err(wasmtime::Trap::new("Unsupported return type")),
						};

						Ok(vec![ret_val])
					},
					Err(e) => {
						// Error handling
						debug!("Host function '{}' returned error: {}", func_name, e);
						Err(wasmtime::Trap::new(format!("Function error: {}", e)))
					},
				}
			};

		// Define the function signature for WASMtime
		let wasmtime_signature = self.wasmtime_signature_from_signature(&func.signature)?;

		// Define the function with the appropriate signature
		let func_type = FuncType::new(self.state_mut().engine(), vec![ValType::I32], vec![ValType::I32]);

		// For now, use a simple wrapper that handles the basic case
		// In production, this would need to properly handle the full signature
		let func_ref = Func::wrap(self.state_mut().engine(), wrapped_callback);

		linker.func_wrap_async(
			"grove",
			&func_name,
			move |mut caller:Caller<'_, T>, _args:Vec<wasmtime::Val>| {
				Box::new(async move {
					// Simple synchronous wrapper for now
					Ok(vec![wasmtime::Val::I32(0)])
				})
			},
		)?;

		Ok(())
	}

	/// Convert our signature to WASMtime signature type
	fn wasmtime_signature_from_signature(&self, sig:&FunctionSignature) -> Result<FuncType> {
		// This is a placeholder - actual implementation depends on the exact types
		// In production, this would map ParamType and ReturnType to WASMtime types
		Ok(FuncType::new(self.registry.bridge.engine(), vec![], vec![]))
	}

	/// Get all registered function names
	pub async fn get_function_names(&self) -> Vec<String> {
		self.registry.functions.read().await.keys().cloned().collect()
	}

	/// Get function statistics
	pub async fn get_function_stats(&self, name:&str) -> Option<FunctionStats> {
		self.registry.functions.read().await.get(name).map(|f| f.stats.clone())
	}

	/// Unregister a function
	#[instrument(skip(self))]
	pub async fn unregister_function(&self, name:&str) -> Result<bool> {
		let mut functions = self.registry.functions.write().await;
		let removed = functions.remove(name).is_some();

		if removed {
			info!("Unregistered host function: {}", name);
		} else {
			warn!("Attempted to unregister non-existent function: {}", name);
		}

		Ok(removed)
	}

	/// Clear all registered functions
	pub async fn clear(&self) {
		info!("Clearing all registered host functions");
		self.registry.functions.write().await.clear();
	}
}

// Placeholder for WASMtime types not directly available
// In actual implementation, these would be imported from wasmtime crate
struct FuncType;
struct ValType;

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_function_export_creation() {
		let bridge = Arc::new(HostBridge::new());
		let export = FunctionExport::new(bridge);

		assert_eq!(export.get_function_names().await.len(), 0);
	}

	#[tokio::test]
	async fn test_register_function() {
		let bridge = Arc::new(HostBridge::new());
		let export = FunctionExport::new(bridge);

		let signature = FunctionSignature {
			name:"echo".to_string(),
			param_types:vec![ParamType::I32],
			return_type:Some(ReturnType::I32),
			is_async:false,
		};

		let callback = |args:Vec<bytes::Bytes>| Ok(args.get(0).cloned().unwrap_or(bytes::Bytes::new()));

		let result = export.register_function("echo", signature, callback).await;
		assert!(result.is_ok());
		assert_eq!(export.get_function_names().await.len(), 1);
	}

	#[tokio::test]
	async fn test_unregister_function() {
		let bridge = Arc::new(HostBridge::new());
		let export = FunctionExport::new(bridge);

		let signature = FunctionSignature {
			name:"test".to_string(),
			param_types:vec![ParamType::I32],
			return_type:Some(ReturnType::I32),
			is_async:false,
		};

		let callback = |_:Vec<bytes::Bytes>| Ok(bytes::Bytes::new());
		export.register_function("test", signature, callback).await.unwrap();

		let result = export.unregister_function("test").await.unwrap();
		assert!(result);
		assert_eq!(export.get_function_names().await.len(), 0);
	}

	#[test]
	fn test_export_config_default() {
		let config = ExportConfig::default();
		assert_eq!(config.auto_export, true);
		assert_eq!(config.max_functions, 1000);
	}

	#[test]
	fn test_function_stats_default() {
		let stats = FunctionStats::default();
		assert_eq!(stats.call_count, 0);
		assert_eq!(stats.error_count, 0);
	}
}
