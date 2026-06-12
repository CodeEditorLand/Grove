//! # Common — Shared Utilities, Traits, and Error Types
//!
//! Provides error types, shared traits, and utility functions used across
//! the Grove codebase.

#[path = "Error.rs"]
pub mod Error;

#[path = "Traits.rs"]
pub mod Traits;

/// Common result type for Grove operations.
pub type Result<T> = anyhow::Result<T>;

/// Grove library version (from `CARGO_PKG_VERSION`).
pub const VERSION:&str = env!("CARGO_PKG_VERSION");

/// Default configuration values.
pub mod config {

	/// Default timeout for operations in milliseconds.
	pub const DEFAULT_TIMEOUT_MS:u64 = 30000;

	/// Default buffer size for I/O operations (8 KiB).
	pub const DEFAULT_BUFFER_SIZE:usize = 8192;

	/// Default maximum number of retries.
	pub const DEFAULT_MAX_RETRIES:u32 = 3;

	/// Default connection timeout in milliseconds.
	pub const DEFAULT_CONNECTION_TIMEOUT_MS:u64 = 5000;

	/// Default heartbeat interval in seconds.
	pub const DEFAULT_HEARTBEAT_INTERVAL_SEC:u64 = 30;

	/// Default maximum concurrent operations.
	pub const DEFAULT_MAX_CONCURRENT:usize = 100;
}

/// Utility functions for common operations.
pub mod utils {

	use std::time::{SystemTime, UNIX_EPOCH};

	/// Returns the current Unix timestamp in seconds.
	pub fn now_unix_timestamp() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() }

	/// Returns the current Unix timestamp in milliseconds.
	pub fn now_unix_timestamp_ms() -> u128 {
		SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
	}

	/// Returns the current Unix timestamp in microseconds.
	pub fn now_unix_timestamp_us() -> u128 {
		SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_micros()
	}

	/// Generates a unique ID string (timestamp + UUID v4).
	pub fn generate_id() -> String { format!("{}-{}", now_unix_timestamp_ms(), uuid::Uuid::new_v4()) }

	/// Sleeps for the specified duration in milliseconds.
	pub async fn sleep_ms(ms:u64) { tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await }

	/// Sleeps for the specified duration in seconds.
	pub async fn sleep_sec(sec:u64) { tokio::time::sleep(tokio::time::Duration::from_secs(sec)).await }
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_version() {
		assert!(!VERSION.is_empty());

		assert!(VERSION.contains('.'));
	}

	#[test]
	fn test_now_unix_timestamp() {
		let ts = utils::now_unix_timestamp();

		assert!(ts > 0);
	}

	#[test]
	fn test_generate_id() {
		let id1 = utils::generate_id();

		let id2 = utils::generate_id();

		assert_ne!(id1, id2);

		assert!(id1.contains('-'));
	}
}
