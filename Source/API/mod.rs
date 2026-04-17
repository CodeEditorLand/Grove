//! API Module
//!
//! Provides the VS Code API facade and types for Grove.
//! Implements compatible API surface with Cocoon for extension compatibility.

#[path = "Types.rs"]
pub mod Types;
#[path = "VSCode.rs"]
pub mod VSCode;

// Re-exports for convenience
pub use Types::*;
pub use VSCode::*;

/// VS Code API version compatibility
pub const VS_CODE_API_VERSION:&str = "1.85.0";

/// Minimum supported VS Code API version
pub const MIN_VS_CODE_API_VERSION:&str = "1.80.0";

/// Maximum supported VS Code API version
pub const MAX_VS_CODE_API_VERSION:&str = "1.90.0";

/// Check if an API version is supported
pub fn is_api_version_supported(version:&str) -> bool {
	match version.parse::<semver::Version>() {
		Ok(v) => {
			let min = MIN_VS_CODE_API_VERSION.parse::<semver::Version>().unwrap();
			let max = MAX_VS_CODE_API_VERSION.parse::<semver::Version>().unwrap();
			v >= min && v <= max
		},
		Err(_) => false,
	}
}

/// Common VS Code API utilities
pub mod utils {
	/// Convert a JSON Value to a specific type
	pub fn from_json_value<T:serde::de::DeserializeOwned>(value:&serde_json::Value) -> Result<T, String> {
		serde_json::from_value(value.clone()).map_err(|e| format!("Failed to deserialize JSON value: {}", e))
	}

	/// Convert a value to a JSON Value
	pub fn to_json_value<T:serde::Serialize>(value:&T) -> Result<serde_json::Value, String> {
		serde_json::to_value(value).map_err(|e| format!("Failed to serialize to JSON value: {}", e))
	}

	/// Check if a JSON value is null
	pub fn is_null(value:&serde_json::Value) -> bool { matches!(value, serde_json::Value::Null) }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_api_version_constants() {
		assert!(!VS_CODE_API_VERSION.is_empty());
		assert!(!MIN_VS_CODE_API_VERSION.is_empty());
		assert!(!MAX_VS_CODE_API_VERSION.is_empty());
	}

	#[test]
	fn test_is_api_version_supported() {
		assert!(is_api_version_supported("1.85.0"));
		assert!(is_api_version_supported("1.80.0"));
		assert!(is_api_version_supported("1.90.0"));
		assert!(!is_api_version_supported("1.79.0"));
		assert!(!is_api_version_supported("1.91.0"));
		assert!(!is_api_version_supported("invalid"));
	}

	#[test]
	fn test_json_utils() {
		use serde::{Deserialize, Serialize};

		#[derive(Debug, Serialize, Deserialize, PartialEq)]
		struct TestValue {
			value:i32,
		}

		let value = TestValue { value:42 };
		let json = utils::to_json_value(&value).unwrap();
		assert_eq!(json["value"], 42);

		let recovered:TestValue = utils::from_json_value(&json).unwrap();
		assert_eq!(recovered, value);
	}

	#[test]
	fn test_is_null() {
		assert!(utils::is_null(&serde_json::Value::Null));
		assert!(!utils::is_null(&serde_json::json!(42)));
	}
}
