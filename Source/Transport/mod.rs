//! Transport Layer Module
//!
//! Provides different communication strategies for Grove.
//! Supports gRPC, IPC, and WASM-based transport methods.
//!
//! # Architecture
//!
//! ```text
//! +++++++++++++++++++++++++++++++++++++++++++
//! +          Transport Strategy             +
//! +++++++++++++++++++++++++++++++++++++++++++
//! +  • gRPC - Network-based communication  +
//! +  • IPC   - Local process communication +
//! +  • WASM  - Direct WASM communication   +
//! +++++++++++++++++++++++++++++++++++++++++++
//!           +                    +
//!           ▼                    ▼
//! ++++++++++++++++++++  ++++++++++++++++++++
//! + Mountain/Core    +  +  Extension       +
//! +  (gRPC client)   +  +  Module (WASM)   +
//! ++++++++++++++++++++  ++++++++++++++++++++
//! ```
//!
//! # Key Components
//!
//! - [`Strategy`] - Transport strategy trait
//! - [`GrpcTransport`] - gRPC-based communication
//! - [`IpcTransport`] - Inter-process communication
//! - [`WASMTransport`] - Direct WASM module communication

pub mod GrpcTransport;
pub mod IpcTransport;
pub mod Strategy;
pub mod WASMTransport;

// Re-exports for convenience
use std::time::Duration;

pub use Strategy::{Transport, TransportStats, TransportStrategy, TransportType};
pub use GrpcTransport::GrpcTransport;
pub use IpcTransport::IPCTransportImpl;
pub use WASMTransport::WASMTransportImpl;
use anyhow::Result;

/// Default connection timeout
pub const DEFAULT_CONNECTION_TIMEOUT_MS:u64 = 5000;

/// Default request timeout
pub const DEFAULT_REQUEST_TIMEOUT_MS:u64 = 30000;

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
	/// Connection timeout
	pub connection_timeout:Duration,
	/// Request timeout
	pub request_timeout:Duration,
	/// Maximum number of retries
	pub max_retries:u32,
	/// Retry delay
	pub retry_delay:Duration,
	/// Enable keepalive
	pub keepalive_enabled:bool,
	/// Keepalive interval
	pub keepalive_interval:Duration,
}

impl Default for TransportConfig {
	fn default() -> Self {
		Self {
			connection_timeout:Duration::from_millis(DEFAULT_CONNECTION_TIMEOUT_MS),
			request_timeout:Duration::from_millis(DEFAULT_REQUEST_TIMEOUT_MS),
			max_retries:3,
			retry_delay:Duration::from_millis(1000),
			keepalive_enabled:true,
			keepalive_interval:Duration::from_secs(30),
		}
	}
}

impl TransportConfig {
	/// Create a new transport configuration
	pub fn new() -> Self { Self::default() }

	/// Set the connection timeout
	pub fn with_connection_timeout(mut self, timeout:Duration) -> Self {
		self.connection_timeout = timeout;
		self
	}

	/// Set the request timeout
	pub fn with_request_timeout(mut self, timeout:Duration) -> Self {
		self.request_timeout = timeout;
		self
	}

	/// Set the maximum number of retries
	pub fn with_max_retries(mut self, max_retries:u32) -> Self {
		self.max_retries = max_retries;
		self
	}

	/// Set the retry delay
	pub fn with_retry_delay(mut self, delay:Duration) -> Self {
		self.retry_delay = delay;
		self
	}

	/// Enable or disable keepalive
	pub fn with_keepalive(mut self, enabled:bool) -> Self {
		self.keepalive_enabled = enabled;
		self
	}
}

/// Create a default transport
pub fn create_default_transport() -> Transport { Transport::default() }

/// Create a gRPC transport with the given address
pub fn create_grpc_transport(address:&str) -> Result<Transport> { Ok(Transport::gRPC(GrpcTransport::new(address)?)) }

/// Create an IPC transport
pub fn create_ipc_transport() -> Result<Transport> { Ok(Transport::IPC(IPCTransportImpl::new()?)) }

/// Create a WASM transport with the given configuration
pub fn create_wasm_transport(enable_wasi:bool, memory_limit_mb:u64, max_execution_time_ms:u64) -> Result<Transport> {
	Ok(Transport::WASM(WASMTransportImpl::new(
		enable_wasi,
		memory_limit_mb,
		max_execution_time_ms,
	)?))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_transport_config_default() {
		let config = TransportConfig::default();
		assert_eq!(config.connection_timeout.as_millis(), DEFAULT_CONNECTION_TIMEOUT_MS as u128);
	}

	#[test]
	fn test_transport_config_builder() {
		let config = TransportConfig::default()
			.with_connection_timeout(Duration::from_secs(10))
			.with_max_retries(5);

		assert_eq!(config.connection_timeout.as_secs(), 10);
		assert_eq!(config.max_retries, 5);
	}

	#[test]
	fn test_transport_default() {
		let transport = create_default_transport();
		// Just test that it can be created
		match transport {
			Transport::gRPC(_) | Transport::IPC(_) | Transport::WASM(_) => {},
		}
	}
}
