//! Shared Traits Module
//!
//! Defines common traits used across the Grove codebase.

use serde::{Deserialize, Serialize};

/// Extension context trait for providing extension-specific information
pub trait ExtensionContext: Send + Sync {
	/// Get the extension ID
	fn extension_id(&self) -> &str;

	/// Get the extension version
	fn version(&self) -> &str;

	/// Get the extension publisher
	fn publisher(&self) -> &str;

	/// Get the extension display name
	fn display_name(&self) -> &str;

	/// Get the extension description
	fn description(&self) -> &str;

	/// Check if the extension is in development mode
	fn is_development(&self) -> bool;
}

/// Extension metadata trait for package.json information
pub trait ExtensionMetadata: Send + Sync {
	/// Get the extension name
	fn name(&self) -> &str;

	/// Get the extension publisher
	fn publisher(&self) -> &str;

	/// Get the extension version
	fn version(&self) -> &str;

	/// Get the extension description
	fn description(&self) -> &str;

	/// Get the main entry point
	fn main(&self) -> &str;

	/// Get activation events
	fn activation_events(&self) -> &[String];

	/// Get extension capabilities
	fn capabilities(&self) -> &[String];

	/// Get extension dependencies
	fn dependencies(&self) -> &[String];

	/// Get the engine compatibility
	fn engine(&self) -> &str;
}

/// Result type for Grove operations
pub type GroveResult<T> = Result<T, GroveError>;

/// Grove error type
#[derive(Debug, thiserror::Error)]
pub enum GroveError {
	/// Extension not found
	#[error("Extension not found: {0}")]
	ExtensionNotFound(String),

	/// Extension activation failed
	#[error("Extension activation failed: {0}")]
	ActivationFailed(String),

	/// Extension deactivation failed
	#[error("Extension deactivation failed: {0}")]
	DeactivationFailed(String),

	/// Transport error
	#[error("Transport error: {0}")]
	TransportError(String),

	/// WASM runtime error
	#[error("WASM runtime error: {0}")]
	WASMError(String),

	/// API error
	#[error("API error: {0}")]
	APIError(String),

	/// Configuration error
	#[error("Configuration error: {0}")]
	ConfigurationError(String),

	/// I/O error
	#[error("I/O error: {0}")]
	IoError(#[from] std::io::Error),

	/// Serialization error
	#[error("Serialization error: {0}")]
	SerializationError(String),

	/// Deserialization error
	#[error("Deserialization error: {0}")]
	DeserializationError(String),

	/// Timeout error
	#[error("Operation timed out")]
	Timeout,

	/// Invalid argument
	#[error("Invalid argument: {0}")]
	InvalidArgument(String),

	/// Not implemented
	#[error("Not implemented: {0}")]
	NotImplemented(String),

	/// Generic error
	#[error("{0}")]
	Other(String),
}

/// Identifiable trait for objects with unique IDs
pub trait Identifiable {
	/// Get the unique identifier
	fn id(&self) -> &str;
}

/// Named trait for objects with names
pub trait Named {
	/// Get the name
	fn name(&self) -> &str;
}

/// Configurable trait for objects with configuration
pub trait Configurable {
	/// Configuration type
	type Config;

	/// Configure the object
	fn configure(&mut self, config:Self::Config) -> anyhow::Result<()>;

	/// Get current configuration
	fn config(&self) -> &Self::Config;
}

/// Resettable trait for objects that can be reset
pub trait Resettable {
	/// Reset the object to its initial state
	fn reset(&mut self) -> anyhow::Result<()>;
}

/// Disposable trait for objects with cleanup
pub trait Disposable {
	/// Dispose and cleanup resources
	fn dispose(&mut self) -> anyhow::Result<()>;
}

/// Cloneable trait for objects that can be cloned with context
pub trait ContextClone {
	/// Clone the object with additional context
	fn clone_with_context(&self, context:&serde_json::Value) -> anyhow::Result<Self>
	where
		Self: Sized;
}

/// Stateful trait for objects with state
pub trait Stateful {
	/// State type
	type State: Clone;

	/// Get current state
	fn state(&self) -> Self::State;

	/// Set state
	fn set_state(&mut self, state:Self::State) -> anyhow::Result<()>;

	/// Restore state
	fn restore_state(&mut self, state:Self::State) -> anyhow::Result<()>;
}

/// Observable trait for objects that can emit events
pub trait Observable {
	/// Event type
	type Event;

	/// Subscribe to events
	fn subscribe(&self, callback:fn(Self::Event)) -> anyhow::Result<()>;

	/// Unsubscribe from events
	fn unsubscribe(&self) -> anyhow::Result<()>;
}

/// Validation trait for objects that can be validated
pub trait Validatable {
	/// Validate the object
	fn validate(&self) -> anyhow::Result<()>;
}

/// Serializable trait for objects that can be serialized
pub trait Serializable: Serialize + for<'de> Deserialize<'de> {
	/// Serialize to JSON string
	fn to_json(&self) -> anyhow::Result<String> {
		serde_json::to_string(self).map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))
	}

	/// Serialize to JSON pretty string
	fn to_json_pretty(&self) -> anyhow::Result<String> {
		serde_json::to_string_pretty(self).map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))
	}

	/// Deserialize from JSON string
	fn from_json(json:&str) -> anyhow::Result<Self>
	where
		Self: Sized, {
		serde_json::from_str(json).map_err(|e| anyhow::anyhow!("Deserialization failed: {}", e))
	}
}

/// Extend Serializable for serializable types
impl<T> Serializable for T where T: Serialize + for<'de> Deserialize<'de> {}

/// Versioned trait for objects with version information
pub trait Versioned {
	/// Get version
	fn version(&self) -> &str;

	/// Check compatibility with another version
	fn is_compatible_with(&self, other_version:&str) -> bool;
}

/// Retryable trait for operations that can be retried
pub trait Retryable {
	/// Execute with retry
	fn execute_with_retry<F, T, E>(&self, mut operation:F, max_retries:u32, delay_ms:u64) -> anyhow::Result<T>
	where
		F: FnMut() -> Result<T, E> + Send,
		E: std::fmt::Display + Send + 'static,
		T: Send, {
		let mut last_error = None;

		for attempt in 0..=max_retries {
			match operation() {
				Ok(result) => return Ok(result),

				Err(e) => {
					last_error = Some(e.to_string());

					if attempt < max_retries {
						std::thread::sleep(std::time::Duration::from_millis(delay_ms));
					}
				},
			}
		}

		Err(anyhow::anyhow!(
			"Operation failed after {} attempts: {}",
			max_retries + 1,
			last_error.unwrap_or_else(|| "Unknown error".to_string())
		))
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_grove_error_display() {
		let err = GroveError::ExtensionNotFound("test.ext".to_string());

		assert_eq!(err.to_string(), "Extension not found: test.ext");

		let err = GroveError::Timeout;

		assert_eq!(err.to_string(), "Operation timed out");
	}

	#[test]
	fn test_serializable_trait() {
		#[derive(Serialize, Deserialize, PartialEq, Debug)]
		struct TestStruct {
			value:i32,
		}

		let test = TestStruct { value:42 };

		let json = test.to_json().unwrap();

		let deserialized:TestStruct = TestStruct::from_json(&json).unwrap();

		assert_eq!(test, deserialized);
	}

	#[test]
	fn test_retryable_execute_with_retry() {
		let retryable = RetryableTrait;

		let mut attempt_count = 0;

		let result = retryable.execute_with_retry(
			|| {
				attempt_count += 1;

				if attempt_count < 3 { Err("Not ready") } else { Ok("Success") }
			},
			5,
			100,
		);

		assert!(result.is_ok());

		assert_eq!(result.unwrap(), "Success");

		assert_eq!(attempt_count, 3);
	}
}

// Helper struct for testing Retryable
struct RetryableTrait;

impl Retryable for RetryableTrait {}
