//! API Bridge Module
//!
//! Provides the VS Code API bridge for extensions.
//! Implements the VS Code API surface that extensions can call.

use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, instrument, warn};

/// API call request from an extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APICallRequest {
	/// Extension ID
	pub extension_id:String,
	/// API method being called
	pub api_method:String,
	/// Function arguments
	pub arguments:Vec<serde_json::Value>,
	/// Correlation ID for async calls
	pub correlation_id:Option<String>,
}

/// API call response to an extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APICallResponse {
	/// Success flag
	pub success:bool,
	/// Response data
	pub data:Option<serde_json::Value>,
	/// Error message (if failed)
	pub error:Option<String>,
	/// Correlation ID (echoed from request)
	pub correlation_id:Option<String>,
}

/// VS Code API call representation
pub struct APICall {
	/// Extension ID
	extension_id:String,
	/// API method
	api_method:String,
	/// Arguments
	arguments:Vec<serde_json::Value>,
	/// Timestamp
	timestamp:u64,
}

/// API method handler callback
type APIMethodHandler = fn(&str, Vec<serde_json::Value>) -> Result<serde_json::Value>;

/// Async API method handler callback
type AsyncAPIMethodHandler =
	fn(&str, Vec<serde_json::Value>) -> Box<dyn std::future::Future<Output = Result<serde_json::Value>> + Send + Unpin>;

/// API method registration
#[derive(Clone)]
struct APIMethodInfo {
	/// Method name
	name:String,
	/// Description
	description:String,
	/// Parameters schema (JSON Schema)
	parameters:Option<serde_json::Value>,
	/// Return type schema (JSON Schema)
	returns:Option<serde_json::Value>,
	/// Whether this method is async
	is_async:bool,
	/// Call count
	call_count:u64,
	/// Total execution time in microseconds
	total_time_us:u64,
}

/// VS Code API bridge for Grove
pub struct APIBridge {
	/// Registered API methods
	api_methods:Arc<RwLock<HashMap<String, APIMethodInfo>>>,
	/// API call statistics
	stats:Arc<RwLock<APIStats>>,
	/// Active API contexts
	contexts:Arc<RwLock<HashMap<String, APIContext>>>,
}

/// API statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct APIStats {
	/// Total number of API calls
	pub total_calls:u64,
	/// Number of successful calls
	pub successful_calls:u64,
	/// Number of failed calls
	pub failed_calls:u64,
	/// Average latency in microseconds
	pub avg_latency_us:u64,
	/// Number of active contexts
	pub active_contexts:usize,
}

/// API context representing an extension's API session
#[derive(Debug, Clone)]
pub struct APIContext {
	/// Extension ID
	pub extension_id:String,
	/// Context ID
	pub context_id:String,
	/// Workspace folder
	pub workspace_folder:Option<String>,
	/// Active editor
	pub active_editor:Option<String>,
	/// Selection ranges
	pub selections:Vec<Selection>,
	/// Context creation timestamp
	pub created_at:u64,
}

/// Text selection range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
	/// Starting line number (0-based)
	pub start_line:u32,
	/// Starting character offset (0-based)
	pub start_character:u32,
	/// Ending line number (0-based)
	pub end_line:u32,
	/// Ending character offset (0-based)
	pub end_character:u32,
}

impl Default for Selection {
	fn default() -> Self { Self { start_line:0, start_character:0, end_line:0, end_character:0 } }
}

impl APIBridge {
	/// Create a new API bridge
	pub fn new() -> Self {
		let bridge = Self {
			api_methods:Arc::new(RwLock::new(HashMap::new())),
			stats:Arc::new(RwLock::new(APIStats::default())),
			contexts:Arc::new(RwLock::new(HashMap::new())),
		};

		bridge.register_builtin_methods();

		bridge
	}

	/// Register built-in VS Code API methods
	fn register_builtin_methods(&self) {
		// This would register all the VS Code API methods
		// For now, we just demonstrate a few examples

		// Example: commands.registerCommand
		// Example: window.showInformationMessage
		// Example: workspace.getConfiguration
		// etc.

		debug!("Registered built-in VS Code API methods");
	}

	/// Register a custom API method
	pub async fn register_method(
		&self,
		name:&str,
		description:&str,
		parameters:Option<serde_json::Value>,
		returns:Option<serde_json::Value>,
		is_async:bool,
	) -> Result<()> {
		let mut methods = self.api_methods.write().await;

		if methods.contains_key(name) {
			warn!("API method already registered: {}", name);
		}

		methods.insert(
			name.to_string(),
			APIMethodInfo {
				name:name.to_string(),
				description:description.to_string(),
				parameters,
				returns,
				is_async,
				call_count:0,
				total_time_us:0,
			},
		);

		debug!("Registered API method: {}", name);

		Ok(())
	}

	/// Create an API context for an extension
	#[instrument(skip(self))]
	pub async fn create_context(&self, extension_id:&str) -> Result<APIContext> {
		let context_id = format!("{}-{}", extension_id, uuid::Uuid::new_v4());

		let context = APIContext {
			extension_id:extension_id.to_string(),
			context_id:context_id.clone(),
			workspace_folder:None,
			active_editor:None,
			selections:Vec::new(),
			created_at:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_secs())
				.unwrap_or(0),
		};

		let mut contexts = self.contexts.write().await;
		contexts.insert(context_id.clone(), context.clone());

		// Update stats
		let mut stats = self.stats.write().await;
		stats.active_contexts = contexts.len();

		debug!("Created API context for extension: {}", extension_id);

		Ok(context)
	}

	/// Get an API context
	pub async fn get_context(&self, context_id:&str) -> Option<APIContext> {
		self.contexts.read().await.get(context_id).cloned()
	}

	/// Update an API context
	pub async fn update_context(&self, context:APIContext) -> Result<()> {
		let mut contexts = self.contexts.write().await;
		contexts.insert(context.context_id.clone(), context);
		Ok(())
	}

	/// Remove an API context
	pub async fn remove_context(&self, context_id:&str) -> Result<bool> {
		let mut contexts = self.contexts.write().await;
		let removed = contexts.remove(context_id).is_some();

		if removed {
			let mut stats = self.stats.write().await;
			stats.active_contexts = contexts.len();
		}

		Ok(removed)
	}

	/// Handle an API call from an extension
	#[instrument(skip(self, request))]
	pub async fn handle_call(&self, request:APICallRequest) -> Result<APICallResponse> {
		let start = std::time::Instant::now();

		debug!(
			"Handling API call: {} from extension {}",
			request.api_method, request.extension_id
		);

		// Check if method exists
		let exists = {
			let methods = self.api_methods.read().await;
			methods.contains_key(&request.api_method)
		};

		if !exists {
			return Ok(APICallResponse {
				success:false,
				data:None,
				error:Some(format!("API method not found: {}", request.api_method)),
				correlation_id:request.correlation_id,
			});
		}

		// Execute the API method (in real implementation, this would call the actual
		// handler)
		let result = self
			.execute_method(&request.extension_id, &request.api_method, &request.arguments)
			.await;

		let elapsed_us = start.elapsed().as_micros() as u64;

		// Update statistics
		let mut stats = self.stats.write().await;
		stats.total_calls += 1;
		stats.total_calls += 1;
		if exists {
			stats.successful_calls += 1;
			// Update average latency
			stats.avg_latency_us =
				(stats.avg_latency_us * (stats.successful_calls - 1) + elapsed_us) / stats.successful_calls;
		}

		// Update method statistics
		{
			let mut methods = self.api_methods.write().await;
			if let Some(method) = methods.get_mut(&request.api_method) {
				method.call_count += 1;
				method.total_time_us += elapsed_us;
			}
		}

		debug!("API call {} completed in {}µs", request.api_method, elapsed_us);

		match result {
			Ok(data) => {
				Ok(
					APICallResponse {
						success:true,
						data:Some(data),
						error:None,
						correlation_id:request.correlation_id,
					},
				)
			},
			Err(e) => {
				Ok(APICallResponse {
					success:false,
					data:None,
					error:Some(e.to_string()),
					correlation_id:request.correlation_id,
				})
			},
		}
	}

	/// Execute an API method
	async fn execute_method(
		&self,
		_extension_id:&str,
		_method_name:&str,
		_arguments:&[serde_json::Value],
	) -> Result<serde_json::Value> {
		// In real implementation, this would:
		// 1. Look up the method handler
		// 2. Validate arguments against schema
		// 3. Call the handler
		// 4. Handle async methods
		// 5. Return the result

		// Placeholder implementation
		Ok(serde_json::Value::Null)
	}

	/// Get API statistics
	pub async fn stats(&self) -> APIStats { self.stats.read().await.clone() }

	/// Get registered API methods
	pub async fn get_methods(&self) -> Vec<APIMethodInfo> { self.api_methods.read().await.values().cloned().collect() }

	/// Unregister an API method
	pub async fn unregister_method(&self, name:&str) -> Result<bool> {
		let mut methods = self.api_methods.write().await;
		let removed = methods.remove(name).is_some();

		if removed {
			debug!("Unregistered API method: {}", name);
		}

		Ok(removed)
	}
}

impl Default for APIBridge {
	fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_api_bridge_creation() {
		let bridge = APIBridge::new();
		let stats = bridge.stats().await;
		assert_eq!(stats.total_calls, 0);
		assert_eq!(stats.successful_calls, 0);
	}

	#[tokio::test]
	async fn test_context_creation() {
		let bridge = APIBridge::new();
		let context = bridge.create_context("test.ext").await.unwrap();
		assert_eq!(context.extension_id, "test.ext");
		assert!(!context.context_id.is_empty());
	}

	#[tokio::test]
	async fn test_method_registration() {
		let bridge = APIBridge::new();
		let result = bridge.register_method("test.method", "Test method", None, None, false).await;
		assert!(result.is_ok());

		let methods = bridge.get_methods().await;
		assert!(methods.iter().any(|m| m.name == "test.method"));
	}

	#[tokio::test]
	async fn test_api_call_request() {
		let request = APICallRequest {
			extension_id:"test.ext".to_string(),
			api_method:"test.method".to_string(),
			arguments:vec![serde_json::json!("arg1")],
			correlation_id:Some("test-id".to_string()),
		};

		assert_eq!(request.extension_id, "test.ext");
		assert_eq!(request.api_method, "test.method");
		assert_eq!(request.arguments.len(), 1);
	}

	#[test]
	fn test_selection_default() {
		let selection = Selection::default();
		assert_eq!(selection.start_line, 0);
		assert_eq!(selection.end_line, 0);
	}
}
