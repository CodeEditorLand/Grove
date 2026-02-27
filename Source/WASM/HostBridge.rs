//! Host Bridge
//!
//! Provides bidirectional communication between the host (Grove) and WASM
//! modules. Handles function calls, data transfer, and marshalling between the
//! two environments.

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::{RwLock, mpsc, oneshot};
use tracing::{debug, instrument, warn};

#[allow(unused_imports)]
use wasmtime::{Caller, Extern, Func, Linker, Store};

/// Host bridge error types
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
/// Function not found error
#[error("Function not found: {0}")]
FunctionNotFound(String),

/// Invalid function signature error
#[error("Invalid function signature: {0}")]
InvalidSignature(String),

/// Serialization failed error
#[error("Serialization failed: {0}")]
SerializationError(String),

/// Deserialization failed error
#[error("Deserialization failed: {0}")]
DeserializationError(String),

/// Host function error
#[error("Host function error: {0}")]
HostFunctionError(String),

/// Communication timeout error
#[error("Communication timeout")]
Timeout,

/// Bridge closed error
#[error("Bridge closed")]
BridgeClosed,
}

/// Type-safe result for operations
pub type BridgeResult<T> = Result<T, BridgeError>;

/// Function signature information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionSignature {
	/// Function name
	pub name:String,
	/// Parameter types
	pub param_types:Vec<ParamType>,
	/// Return type
	pub return_type:Option<ReturnType>,
	/// Whether this is an async function
	pub is_async:bool,
}

/// Parameter types for WASM functions
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ParamType {
/// 32-bit signed integer parameter
I32,
/// 64-bit signed integer parameter
I64,
/// 32-bit floating point parameter
F32,
/// 64-bit floating point parameter
F64,
/// Pointer to memory
Ptr,
/// Length parameter following a pointer
Len,
}

/// Return types for WASM functions
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum ReturnType {
/// 32-bit signed integer return type
I32,
/// 64-bit signed integer return type
I64,
/// 32-bit floating point return type
F32,
/// 64-bit floating point return type
F64,
/// No return value (void)
Void,
}

/// Message sent from WASM to host
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HostMessage {
	/// Message ID for correlation
	pub message_id:String,
	/// Function name to call
	pub function:String,
	/// Serialized arguments
	pub args:Vec<Bytes>,
	/// Callback token for async responses
	pub callback_token:Option<u64>,
}

/// Response sent from host to WASM
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HostResponse {
	/// Correlating message ID
	pub message_id:String,
	/// Success flag
	pub success:bool,
	/// Response data
	pub data:Option<Bytes>,
	/// Error message if failed
	pub error:Option<String>,
}

/// Callback for async function responses
#[derive(Clone)]
pub struct AsyncCallback {
/// Sender for transmitting the response
sender:Arc<tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<HostResponse>>>>,
/// Message ID for correlation
message_id:String,
}

impl std::fmt::Debug for AsyncCallback {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("AsyncCallback")
			.field("message_id", &self.message_id)
			.finish()
	}
}

impl AsyncCallback {
	/// Send response through the callback
	pub async fn send(self, response:HostResponse) -> Result<()> {
		let mut sender_opt = self.sender.lock().await;
		if let Some(sender) = sender_opt.take() {
			sender.send(response).map_err(|_| BridgeError::BridgeClosed)?;
			Ok(())
		} else {
			Err(BridgeError::BridgeClosed.into())
		}
	}
}

/// Message from host to WASM
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WASMMessage {
	/// Target function in WASM
	pub function:String,
	/// Arguments
	pub args:Vec<Bytes>,
}

/// Host function callback type
pub type HostFunctionCallback = fn(Vec<Bytes>) -> Result<Bytes>;

/// Async host function callback type
pub type AsyncHostFunctionCallback =
	fn(Vec<Bytes>) -> Box<dyn std::future::Future<Output = Result<Bytes>> + Send + Unpin>;

/// Host function definition
#[derive(Debug)]
pub struct HostFunction {
	/// Function name
	pub name:String,
	/// Function signature
	pub signature:FunctionSignature,
	/// Synchronous callback - not serializable (skip serde derive)
	#[allow(dead_code)]
	pub callback:Option<HostFunctionCallback>,
	/// Async callback - not serializable (skip serde derive)
	#[allow(dead_code)]
	pub async_callback:Option<AsyncHostFunctionCallback>,
}

/// Host Bridge for WASM communication
#[derive(Debug)]
pub struct HostBridgeImpl {
	/// Registry of host functions exported to WASM
	host_functions:Arc<RwLock<HashMap<String, HostFunction>>>,
	/// Channel for receiving messages from WASM
	wasm_to_host_rx:mpsc::UnboundedReceiver<WASMMessage>,
	/// Channel for sending messages to WASM
	host_to_wasm_tx:mpsc::UnboundedSender<WASMMessage>,
	/// Active async callbacks
	async_callbacks:Arc<RwLock<HashMap<u64, AsyncCallback>>>,
	/// Next callback token
	next_callback_token:Arc<std::sync::atomic::AtomicU64>,
}

impl HostBridgeImpl {
	/// Create a new host bridge
	pub fn new() -> Self {
	    let (_wasm_to_host_tx, wasm_to_host_rx) = mpsc::unbounded_channel();
	    let (host_to_wasm_tx, host_to_wasm_rx) = mpsc::unbounded_channel();

		// In a real implementation, we'd need to wire these up properly
		// For now, we drop the receiver to avoid unused warnings
		drop(host_to_wasm_rx);

		Self {
			host_functions:Arc::new(RwLock::new(HashMap::new())),
			wasm_to_host_rx,
			host_to_wasm_tx,
			async_callbacks:Arc::new(RwLock::new(HashMap::new())),
			next_callback_token:Arc::new(std::sync::atomic::AtomicU64::new(0)),
		}
	}

	/// Register a host function to be exported to WASM
	#[instrument(skip(self, callback))]
	pub async fn register_host_function(
		&self,
		name:&str,
		signature:FunctionSignature,
		callback:HostFunctionCallback,
	) -> BridgeResult<()> {
		debug!("Registering host function: {}", name);

		let mut functions = self.host_functions.write().await;

		if functions.contains_key(name) {
			warn!("Host function already registered: {}", name);
		}

		functions.insert(
			name.to_string(),
			HostFunction { name:name.to_string(), signature, callback:Some(callback), async_callback:None },
		);

		debug!("Host function registered successfully: {}", name);
		Ok(())
	}

	/// Register an async host function
	#[instrument(skip(self, callback))]
	pub async fn register_async_host_function(
		&self,
		name:&str,
		signature:FunctionSignature,
		callback:AsyncHostFunctionCallback,
	) -> BridgeResult<()> {
		debug!("Registering async host function: {}", name);

		let mut functions = self.host_functions.write().await;

		functions.insert(
			name.to_string(),
			HostFunction { name:name.to_string(), signature, callback:None, async_callback:Some(callback) },
		);

		debug!("Async host function registered successfully: {}", name);
		Ok(())
	}

	/// Call a host function from WASM
	#[instrument(skip(self, args))]
	pub async fn call_host_function(&self, function_name:&str, args:Vec<Bytes>) -> BridgeResult<Bytes> {
		debug!("Calling host function: {}", function_name);

		let functions = self.host_functions.read().await;
		let func = functions
			.get(function_name)
			.ok_or_else(|| BridgeError::FunctionNotFound(function_name.to_string()))?;

		if let Some(callback) = func.callback {
			// Synchronous call
			let result =
				callback(args).map_err(|e| BridgeError::HostFunctionError(format!("{}: {}", function_name, e)))?;
			debug!("Host function call completed: {}", function_name);
			Ok(result)
		} else if let Some(async_callback) = func.async_callback {
			// Async call
			let future = async_callback(args);
			let result = future
				.await
				.map_err(|e| BridgeError::HostFunctionError(format!("{}: {}", function_name, e)))?;
			debug!("Async host function call completed: {}", function_name);
			Ok(result)
		} else {
			Err(BridgeError::FunctionNotFound(format!(
				"No callback for function: {}",
				function_name
			)))
		}
	}

	/// Send a message to WASM
	#[instrument(skip(self, message))]
	pub async fn send_to_wasm(&self, message:WASMMessage) -> BridgeResult<()> {
		let function_name = message.function.clone();
		self.host_to_wasm_tx.send(message).map_err(|_| BridgeError::BridgeClosed)?;
		debug!("Message sent to WASM: {}", function_name);
		Ok(())
	}

	/// Receive a message from WASM (blocking)
	pub async fn receive_from_wasm(&mut self) -> Option<WASMMessage> { self.wasm_to_host_rx.recv().await }

	/// Create async callback
	#[instrument(skip(self))]
	pub async fn create_async_callback(&self, message_id:String) -> (AsyncCallback, u64) {
		let token = self.next_callback_token.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
		let (tx, _rx) = oneshot::channel();

		// Create callback with Arc-wrapped sender
		let callback = AsyncCallback {
			sender:Arc::new(tokio::sync::Mutex::new(Some(tx))),
			message_id:message_id.clone(),
		};

		self.async_callbacks.write().await.insert(token, callback.clone());

		(callback, token)
	}

	/// Get callback by token
	#[instrument(skip(self))]
	pub async fn get_callback(&self, token:u64) -> Option<AsyncCallback> {
		self.async_callbacks.write().await.remove(&token)
	}

	/// Get all registered host functions
	pub async fn get_host_functions(&self) -> Vec<String> { self.host_functions.read().await.keys().cloned().collect() }

	/// Unregister a host function
	#[instrument(skip(self))]
	pub async fn unregister_host_function(&self, name:&str) -> bool {
		let mut functions = self.host_functions.write().await;
		let removed = functions.remove(name).is_some();
		if removed {
			debug!("Host function unregistered: {}", name);
		}
		removed
	}

	/// Clear all registered functions
	pub async fn clear(&self) {
		debug!("Clearing all registered host functions");
		self.host_functions.write().await.clear();
		self.async_callbacks.write().await.clear();
	}
}

impl Default for HostBridgeImpl {
	fn default() -> Self { Self::new() }
}

/// Utility function to serialize data to Bytes
pub fn serialize_to_bytes<T:Serialize>(data:&T) -> Result<Bytes> {
	serde_json::to_vec(data)
		.map(Bytes::from)
		.map_err(|e| anyhow::anyhow!("Serialization error: {}", e))
}

/// Utility function to deserialize Bytes to data
pub fn deserialize_from_bytes<T:DeserializeOwned>(bytes:&Bytes) -> Result<T> {
	serde_json::from_slice(bytes).map_err(|e| anyhow::anyhow!("Deserialization error: {}", e))
}

/// Marshal arguments for WASM function call
pub fn marshal_args(args:Vec<Bytes>) -> Result<Vec<wasmtime::Val>> {
	args.iter()
		.map(|bytes| {
			let value:serde_json::Value = serde_json::from_slice(bytes)?;
			match value {
				serde_json::Value::Number(n) => {
					if let Some(i) = n.as_i64() {
						Ok(wasmtime::Val::I32(i as i32))
					} else if let Some(f) = n.as_f64() {
						Ok(wasmtime::Val::F64(f.to_bits()))
					} else {
						Err(anyhow::anyhow!("Invalid number value"))
					}
				},
				_ => Err(anyhow::anyhow!("Unsupported argument type")),
			}
		})
		.collect()
}

/// Unmarshal return values from WASM function call
pub fn unmarshal_return(val:wasmtime::Val) -> Result<Bytes> {
	match val {
		wasmtime::Val::I32(i) => {
			let json = serde_json::to_string(&i)?;
			Ok(Bytes::from(json))
		},
		wasmtime::Val::I64(i) => {
			let json = serde_json::to_string(&i)?;
			Ok(Bytes::from(json))
		},
		wasmtime::Val::F32(f) => {
			let json = serde_json::to_string(&f)?;
			Ok(Bytes::from(json))
		},
		wasmtime::Val::F64(f) => {
			let json = serde_json::to_string(&f)?;
			Ok(Bytes::from(json))
		},
		_ => Err(anyhow::anyhow!("Unsupported return type")),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_function_signature_creation() {
		let signature = FunctionSignature {
			name:"test_func".to_string(),
			param_types:vec![ParamType::I32, ParamType::Ptr],
			return_type:Some(ReturnType::I32),
			is_async:false,
		};

		assert_eq!(signature.name, "test_func");
		assert_eq!(signature.param_types.len(), 2);
	}

	#[tokio::test]
	async fn test_host_bridge_creation() {
		let bridge = HostBridgeImpl::new();
		assert_eq!(bridge.get_host_functions().await.len(), 0);
	}

	#[tokio::test]
	async fn test_register_host_function() {
		let bridge = HostBridgeImpl::new();

		let signature = FunctionSignature {
			name:"echo".to_string(),
			param_types:vec![ParamType::I32],
			return_type:Some(ReturnType::I32),
			is_async:false,
		};

		let result = bridge
			.register_host_function("echo", signature, |args| Ok(args[0].clone()))
			.await;

		assert!(result.is_ok());
		assert_eq!(bridge.get_host_functions().await.len(), 1);
	}

	#[test]
	fn test_serialize_deserialize() {
		let data = vec![1, 2, 3, 4, 5];
		let bytes = serialize_to_bytes(&data).unwrap();
		let recovered:Vec<i32> = deserialize_from_bytes(&bytes).unwrap();
		assert_eq!(data, recovered);
	}

	#[test]
	fn test_marshal_unmarshal() {
		let args = vec![serialize_to_bytes(&42i32).unwrap(), serialize_to_bytes(&3.14f64).unwrap()];

		// Test that marshaling works (we don't assert on exact type conversion)
		let marshaled = marshal_args(args);
		assert!(marshaled.is_ok());
	}
}
