//! # Shared Traits
//!
//! Defines common traits used across the Grove codebase, including extension
//! metadata, configuration, lifecycle, serialization, and observability.

use serde::{Deserialize, Serialize};

/// Provides extension-specific context information.
pub trait ExtensionContext: Send + Sync {
	/// Returns the extension ID.
	fn extension_id(&self) -> &str;

	/// Returns the extension version.
	fn version(&self) -> &str;

	/// Returns the extension publisher.
	fn publisher(&self) -> &str;

	/// Returns the extension display name.
	fn display_name(&self) -> &str;

	/// Returns the extension description.
	fn description(&self) -> &str;

	/// Returns `true` if the extension is in development mode.
	fn is_development(&self) -> bool;
}

/// Provides metadata from an extension's `package.json` manifest.
pub trait ExtensionMetadata: Send + Sync {
	/// Returns the extension name.
	fn name(&self) -> &str;

	/// Returns the extension publisher.
	fn publisher(&self) -> &str;

	/// Returns the extension version.
	fn version(&self) -> &str;

	/// Returns the extension description.
	fn description(&self) -> &str;

	/// Returns the main entry point path.
	fn main(&self) -> &str;

	/// Returns the activation events.
	fn activation_events(&self) -> &[String];

	/// Returns the extension capabilities.
	fn capabilities(&self) -> &[String];

	/// Returns the extension dependencies.
	fn dependencies(&self) -> &[String];

	/// Returns the engine compatibility string.
	fn engine(&self) -> &str;
}

/// Result type for Grove operations.
pub type GroveResult<T> = Result<T, GroveError>;

/// Grove error type.
#[derive(Debug, thiserror::Error)]
pub enum GroveError {
	/// Extension not found.
	#[error("Extension not found: {0}")]
	ExtensionNotFound(String),

	/// Extension activation failed.
	#[error("Extension activation failed: {0}")]
	ActivationFailed(String),

	/// Extension deactivation failed.
	#[error("Extension deactivation failed: {0}")]
	DeactivationFailed(String),

	/// Transport error.
	#[error("Transport error: {0}")]
	TransportError(String),

	/// WASM runtime error.
	#[error("WASM runtime error: {0}")]
	WASMError(String),

	/// API error.
	#[error("API error: {0}")]
	APIError(String),

	/// Configuration error.
	#[error("Configuration error: {0}")]
	ConfigurationError(String),

	/// I/O error.
	#[error("I/O error: {0}")]
	IoError(#[from] std::io::Error),

	/// Serialization error.
	#[error("Serialization error: {0}")]
	SerializationError(String),

	/// Deserialization error.
	#[error("Deserialization error: {0}")]
	DeserializationError(String),

	/// Operation timed out.
	#[error("Operation timed out")]
	Timeout,

	/// Invalid argument.
	#[error("Invalid argument: {0}")]
	InvalidArgument(String),

	/// Not implemented.
	#[error("Not implemented: {0}")]
	NotImplemented(String),

	/// Generic error.
	#[error("{0}")]
	Other(String),
}

/// Trait for objects with a unique identifier.
pub trait Identifiable {
	/// Returns the unique identifier.
	fn id(&self) -> &str;
}

/// Trait for objects with a name.
pub trait Named {
	/// Returns the name.
	fn name(&self) -> &str;
}

/// Trait for objects that can be configured.
pub trait Configurable {
	/// The configuration type.
	type Config;

	/// Configures the object with the given configuration.
	fn configure(&mut self, config: Self::Config) -> anyhow::Result<()>;

	/// Returns a reference to the current configuration.
	fn config(&self) -> &Self::Config;
}

/// Trait for objects that can be reset to their initial state.
pub trait Resettable {
	/// Resets the object to its initial state.
	fn reset(&mut self) -> anyhow::Result<()>;
}

/// Trait for objects that require cleanup when disposed.
pub trait Disposable {
	/// Disposes of the object, releasing resources.
	fn dispose(&mut self) -> anyhow::Result<()>;
}

/// Trait for objects that can be cloned with additional context.
pub trait ContextClone {
	/// Clones the object with additional context data.
	fn clone_with_context(&self, context: &serde_json::Value) -> anyhow::Result<Self>
	where
		Self: Sized;
}

/// Trait for objects with observable state.
pub trait Stateful {
	/// The state type.
	type State: Clone;

	/// Returns the current state.
	fn state(&self) -> Self::State;

	/// Sets the state.
	fn set_state(&mut self, state: Self::State) -> anyhow::Result<()>;

	/// Restores the state from a previous snapshot.
	fn restore_state(&mut self, state: Self::State) -> anyhow::Result<()>;
}

/// Trait for objects that can emit events.
pub trait Observable {
	/// The event type.
	type Event;

	/// Subscribes to events with the given callback.
	fn subscribe(&self, callback: fn(Self::Event)) -> anyhow::Result<()>;

	/// Unsubscribes from all events.
	fn unsubscribe(&self) -> anyhow::Result<()>;
}

/// Trait for objects that can be validated.
pub trait Validatable {
	/// Validates the object, returning an error if invalid.
	fn validate(&self) -> anyhow::Result<()>;
}

/// Trait for objects that can be serialized to/from JSON.
pub trait Serializable: Serialize + for<'de> Deserialize<'de> {
	/// Serializes to a JSON string.
	fn to_json(&self) -> anyhow::Result<String> {
		serde_json::to_string(self).map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))
	}

	/// Serializes to a pretty-printed JSON string.
	fn to_json_pretty(&self) -> anyhow::Result<String> {
		serde_json::to_string_pretty(self).map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))
	}

	/// Deserializes from a JSON string.
	fn from_json(json: &str) -> anyhow::Result<Self>
	where
		Self: Sized,
	{
		serde_json::from_str(json).map_err(|e| anyhow::anyhow!("Deserialization failed: {}", e))
	}
}

/// Blanket implementation of `Serializable` for all `Serialize + Deserialize` types.
impl<T> Serializable for T where T: Serialize + for<'de> Deserialize<'de> {}

/// Trait for objects with version information.
pub trait Versioned {
	/// Returns the version string.
	fn version(&self) -> &str;

	/// Returns `true` if this version is compatible with the given version.
	fn is_compatible_with(&self, other_version: &str) -> bool;
}

/// Trait for operations that can be retried on failure.
pub trait Retryable {
	/// Executes an operation with retry logic.
	///
	/// ## Parameters
	///
	/// * `operation` — The operation to execute.
	/// * `max_retries` — Maximum number of retry attempts.
	/// * `delay_ms` — Delay between retries in milliseconds.
	///
	/// ## Returns
	///
	/// The operation result, or an error after all retries are exhausted.
	fn execute_with_retry<F, T, E>(&self, mut operation: F, max_retries: u32, delay_ms: u64) -> anyhow::Result<T>
	where
		F: FnMut() -> Result<T, E> + Send,
		E: std::fmt::Display + Send + 'static,
		T: Send,
	{
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
			value: i32,
		}

		let test = TestStruct { value: 42 };

		let json = test.to_json().unwrap();

		let deserialized: TestStruct = TestStruct::from_json(&json).unwrap();

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
