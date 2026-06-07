//! Error Types Module
//!
//! Defines error types used throughout the Grove codebase.
//! This module provides a unified error handling approach.

use std::fmt;

/// Grove result type alias
pub type GroveResult<T> = Result<T, GroveError>;

/// Grove error type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GroveError {

	/// Extension not found error
	ExtensionNotFound {
		/// The extension identifier
		extension_id:String,

		/// Optional error message
		message:Option<String>,
	},

	/// Extension loading failed error
	ExtensionLoadFailed {
		/// The extension identifier
		extension_id:String,

		/// The failure reason
		reason:String,

		/// Optional path to the extension
		path:Option<String>,
	},

	/// Extension activation failed error
	ActivationFailed {
		/// The extension identifier
		extension_id:String,

		/// The failure reason
		reason:String,
	},

	/// Extension deactivation failed error
	DeactivationFailed {
		/// The extension identifier
		extension_id:String,

		/// The failure reason
		reason:String,
	},

	/// WASM runtime error
	WASMRuntimeError {
		/// The error reason
		reason:String,

		/// Optional module identifier
		module_id:Option<String>,
	},

	/// WASM compilation failed error
	WASMCompilationFailed {
		/// The failure reason
		reason:String,

		/// Optional path to the module
		module_path:Option<String>,
	},

	/// WASM module not found error
	WASMModuleNotFound {
		/// The module identifier
		module_id:String,
	},

	/// Transport error
	TransportError {
		/// The transport type
		transport_type:String,

		/// The error reason
		reason:String,
	},

	/// Connection error
	ConnectionError {
		/// The endpoint that failed
		endpoint:String,

		/// The error reason
		reason:String,
	},

	/// API call error
	APIError {
		/// The API method that failed
		api_method:String,

		/// The error reason
		reason:String,

		/// Optional error code
		error_code:Option<i32>,
	},

	/// Configuration error
	ConfigurationError {
		/// The configuration key
		key:String,

		/// The error reason
		reason:String,
	},

	/// I/O error
	IoError {
		/// Optional path related to the error
		path:Option<String>,

		/// The operation that failed
		operation:String,

		/// The error reason
		reason:String,
	},

	/// Serialization error
	SerializationError {
		/// The type name being serialized
		type_name:String,

		/// The error reason
		reason:String,
	},

	/// Deserialization error
	DeserializationError {
		/// The type name being deserialized
		type_name:String,

		/// The error reason
		reason:String,
	},

	/// Timeout error
	Timeout {
		/// The operation that timed out
		operation:String,

		/// The timeout duration in milliseconds
		timeout_ms:u64,
	},

	/// Invalid argument error
	InvalidArgument {
		/// The argument name
		argument_name:String,

		/// The error reason
		reason:String,
	},

	/// Not implemented error
	NotImplemented {
		/// The feature that is not implemented
		feature:String,
	},

	/// Permission denied error
	PermissionDenied {
		/// The resource that was denied
		resource:String,

		/// The error reason
		reason:String,
	},

	/// Resource exhausted error
	ResourceExhausted {
		/// The resource that was exhausted
		resource:String,

		/// The error reason
		reason:String,
	},

	/// Internal error
	InternalError {
		/// The error reason
		reason:String,

		/// Optional backtrace (skipped during serialization)
		#[serde(skip)]
		backtrace:Option<String>,
	},
}

impl GroveError {

	/// Create extension not found error
	pub fn extension_not_found(extension_id:impl Into<String>) -> Self {
		Self::ExtensionNotFound { extension_id:extension_id.into(), message:None }
	}

	/// Create extension load failed error
	pub fn extension_load_failed(extension_id:impl Into<String>, reason:impl Into<String>) -> Self {
		Self::ExtensionLoadFailed { extension_id:extension_id.into(), reason:reason.into(), path:None }
	}

	/// Create activation failed error
	pub fn activation_failed(extension_id:impl Into<String>, reason:impl Into<String>) -> Self {
		Self::ActivationFailed { extension_id:extension_id.into(), reason:reason.into() }
	}

	/// Create WASM runtime error
	pub fn wasm_runtime_error(reason:impl Into<String>) -> Self {
		Self::WASMRuntimeError { reason:reason.into(), module_id:None }
	}

	/// Create transport error
	pub fn transport_error(transport_type:impl Into<String>, reason:impl Into<String>) -> Self {
		Self::TransportError { transport_type:transport_type.into(), reason:reason.into() }
	}

	/// Create connection error
	pub fn connection_error(endpoint:impl Into<String>, reason:impl Into<String>) -> Self {
		Self::ConnectionError { endpoint:endpoint.into(), reason:reason.into() }
	}

	/// Create API error
	pub fn api_error(api_method:impl Into<String>, reason:impl Into<String>) -> Self {
		Self::APIError { api_method:api_method.into(), reason:reason.into(), error_code:None }
	}

	/// Create timeout error
	pub fn timeout(operation:impl Into<String>, timeout_ms:u64) -> Self {
		Self::Timeout { operation:operation.into(), timeout_ms }
	}

	/// Create invalid argument error
	pub fn invalid_argument(argument_name:impl Into<String>, reason:impl Into<String>) -> Self {
		Self::InvalidArgument { argument_name:argument_name.into(), reason:reason.into() }
	}

	/// Create not implemented error
	pub fn not_implemented(feature:impl Into<String>) -> Self { Self::NotImplemented { feature:feature.into() } }

	/// Get error code for categorization
	pub fn error_code(&self) -> &'static str {
		match self {
			Self::ExtensionNotFound { .. } => "EXT_NOT_FOUND",

			Self::ExtensionLoadFailed { .. } => "EXT_LOAD_FAILED",

			Self::ActivationFailed { .. } => "ACTIVATION_FAILED",

			Self::DeactivationFailed { .. } => "DEACTIVATION_FAILED",

			Self::WASMRuntimeError { .. } => "WASM_RUNTIME_ERROR",

			Self::WASMCompilationFailed { .. } => "WASM_COMPILATION_FAILED",

			Self::WASMModuleNotFound { .. } => "WASM_MODULE_NOT_FOUND",

			Self::TransportError { .. } => "TRANSPORT_ERROR",

			Self::ConnectionError { .. } => "CONNECTION_ERROR",

			Self::APIError { .. } => "API_ERROR",

			Self::ConfigurationError { .. } => "CONFIGURATION_ERROR",

			Self::IoError { .. } => "IO_ERROR",

			Self::SerializationError { .. } => "SERIALIZATION_ERROR",

			Self::DeserializationError { .. } => "DESERIALIZATION_ERROR",

			Self::Timeout { .. } => "TIMEOUT",

			Self::InvalidArgument { .. } => "INVALID_ARGUMENT",

			Self::NotImplemented { .. } => "NOT_IMPLEMENTED",

			Self::PermissionDenied { .. } => "PERMISSION_DENIED",

			Self::ResourceExhausted { .. } => "RESOURCE_EXHAUSTED",

			Self::InternalError { .. } => "INTERNAL_ERROR",
		}
	}

	/// Check if error is recoverable
	pub fn is_recoverable(&self) -> bool {
		matches!(
			self,

			Self::Timeout { .. }
				| Self::TransportError { .. }
				| Self::ConnectionError { .. }
				| Self::ResourceExhausted { .. }
		)
	}

	/// Check if error is transient (can be retried)
	pub fn is_transient(&self) -> bool {
		matches!(
			self,

			Self::Timeout { .. } | Self::TransportError { .. } | Self::ConnectionError { .. }
		)
	}
}

impl fmt::Display for GroveError {

	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::ExtensionNotFound { extension_id, message } => {
				if let Some(msg) = message {
					write!(f, "Extension not found: {} - {}", extension_id, msg)
				} else {
					write!(f, "Extension not found: {}", extension_id)
				}
			},

			Self::ExtensionLoadFailed { extension_id, reason, path } => {
				if let Some(path) = path {
					write!(f, "Failed to load extension #{:?}: {} - {}", path, extension_id, reason)
				} else {
					write!(f, "Failed to load extension {}: {}", extension_id, reason)
				}
			},

			Self::ActivationFailed { extension_id, reason } => {
				write!(f, "Activation failed for extension {}: {}", extension_id, reason)
			},

			Self::DeactivationFailed { extension_id, reason } => {
				write!(f, "Deactivation failed for extension {}: {}", extension_id, reason)
			},

			Self::WASMRuntimeError { reason, module_id } => {
				if let Some(id) = module_id {
					write!(f, "WASM runtime error for module {}: {}", id, reason)
				} else {
					write!(f, "WASM runtime error: {}", reason)
				}
			},

			Self::WASMCompilationFailed { reason, module_path } => {
				if let Some(path) = module_path {
					write!(f, "WASM compilation failed for {:?}: {}", path, reason)
				} else {
					write!(f, "WASM compilation failed: {}", reason)
				}
			},

			Self::WASMModuleNotFound { module_id } => {
				write!(f, "WASM module not found: {}", module_id)
			},

			Self::TransportError { transport_type, reason } => {
				write!(f, "Transport error ({:?}): {}", transport_type, reason)
			},

			Self::ConnectionError { endpoint, reason } => {
				write!(f, "Connection error to {}: {}", endpoint, reason)
			},

			Self::APIError { api_method, reason, .. } => {
				write!(f, "API error for {}: {}", api_method, reason)
			},

			Self::ConfigurationError { key, reason } => {
				write!(f, "Configuration error for '{}': {}", key, reason)
			},

			Self::IoError { operation, reason, .. } => {
				write!(f, "I/O error for operation '{}': {}", operation, reason)
			},

			Self::SerializationError { type_name, reason } => {
				write!(f, "Serialization error for type '{}': {}", type_name, reason)
			},

			Self::DeserializationError { type_name, reason } => {
				write!(f, "Deserialization error for type '{}': {}", type_name, reason)
			},

			Self::Timeout { operation, timeout_ms } => {
				write!(f, "Timeout after {}ms for operation: {}", timeout_ms, operation)
			},

			Self::InvalidArgument { argument_name, reason } => {
				write!(f, "Invalid argument '{}': {}", argument_name, reason)
			},

			Self::NotImplemented { feature } => {
				write!(f, "Feature not implemented: {}", feature)
			},

			Self::PermissionDenied { resource, reason } => {
				write!(f, "Permission denied for '{}': {}", resource, reason)
			},

			Self::ResourceExhausted { resource, reason } => {
				write!(f, "Resource exhausted '{}': {}", resource, reason)
			},

			Self::InternalError { reason, .. } => {
				write!(f, "Internal error: {}", reason)
			},
		}
	}
}

impl std::error::Error for GroveError {}

/// Convert from std::io::Error
impl From<std::io::Error> for GroveError {

	fn from(err:std::io::Error) -> Self {
		Self::IoError { path:None, operation:"unknown".to_string(), reason:err.to_string() }
	}
}

/// Convert from serde_json::Error
impl From<serde_json::Error> for GroveError {

	fn from(err:serde_json::Error) -> Self {
		if err.is_io() {
			Self::IoError { path:None, operation:"serde_json".to_string(), reason:err.to_string() }
		} else {
			Self::DeserializationError { type_name:"unknown".to_string(), reason:err.to_string() }
		}
	}
}

/// Result extension trait for error handling
pub trait ResultExt<T> {

	/// Map error to GroveError
	fn map_grove_error(self, context:impl Into<String>) -> GroveResult<T>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
	E: std::error::Error + Send + Sync + 'static,

{

	fn map_grove_error(self, context:impl Into<String>) -> GroveResult<T> {
		self.map_err(|e| {
			GroveError::InternalError {
				reason:format!("{}: {}", context.into(), e),
				backtrace:std::backtrace::Backtrace::capture().to_string().into(),
			}
		})
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_error_creation() {
		let err = GroveError::extension_not_found("test.ext");

		assert_eq!(err.error_code(), "EXT_NOT_FOUND");
	}

	#[test]
	fn test_error_display() {
		let err = GroveError::activation_failed("test.ext", "timeout");

		assert!(err.to_string().contains("test.ext"));

		assert!(err.to_string().contains("timeout"));
	}

	#[test]
	fn test_error_retryable() {
		let timeout = GroveError::timeout("test", 5000);

		assert!(timeout.is_transient());

		assert!(timeout.is_recoverable());

		let not_found = GroveError::extension_not_found("test.ext");

		assert!(!not_found.is_transient());

		assert!(!not_found.is_recoverable());
	}

	#[test]
	fn test_io_error_conversion() {
		let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");

		let grove_err = GroveError::from(io_err);

		assert_eq!(grove_err.error_code(), "IO_ERROR");
	}
}
