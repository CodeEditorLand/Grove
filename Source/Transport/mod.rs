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
//! - [`gRPCTransport`] - gRPC-based communication
//! - [`IPCTransport`] - Inter-process communication
//! - [`WASMTransport`] - Direct WASM module communication

pub mod gRPCTransport;
pub mod IPCTransport;
pub mod Strategy;
pub mod WASMTransport;

use std::time::Duration;

use anyhow::Result;
// Types accessed via full paths: Transport::Strategy::Transport, etc.

/// Default connection timeout
pub const DEFAULT_CONNECTION_TIMEOUT_MS:u64 = 5000;

/// Default request timeout
pub const DEFAULT_REQUEST_TIMEOUT_MS:u64 = 30000;

/// Transport configuration.
#[derive(Debug, Clone)]
pub struct TransportConfig {
	/// Connection timeout.
	pub ConnectionTimeout:Duration,
	/// Request timeout.
	pub RequestTimeout:Duration,
	/// Maximum number of retries.
	pub MaximumRetries:u32,
	/// Delay between retries.
	pub RetryDelay:Duration,
	/// Whether keepalive is enabled.
	pub KeepaliveEnabled:bool,
	/// Keepalive interval.
	pub KeepaliveInterval:Duration,
}

impl Default for TransportConfig {
	fn default() -> Self {
		Self {
			ConnectionTimeout:Duration::from_millis(DEFAULT_CONNECTION_TIMEOUT_MS),
			RequestTimeout:Duration::from_millis(DEFAULT_REQUEST_TIMEOUT_MS),
			MaximumRetries:3,
			RetryDelay:Duration::from_millis(1000),
			KeepaliveEnabled:true,
			KeepaliveInterval:Duration::from_secs(30),
		}
	}
}

impl TransportConfig {
	/// Creates a new `TransportConfig` with default values.
	pub fn New() -> Self { Self::default() }

	/// Sets the connection timeout.
	pub fn WithConnectionTimeout(mut self, Timeout:Duration) -> Self {
		self.ConnectionTimeout = Timeout;
		self
	}

	/// Sets the request timeout.
	pub fn WithRequestTimeout(mut self, Timeout:Duration) -> Self {
		self.RequestTimeout = Timeout;
		self
	}

	/// Sets the maximum number of retries.
	pub fn WithMaximumRetries(mut self, MaximumRetries:u32) -> Self {
		self.MaximumRetries = MaximumRetries;
		self
	}

	/// Sets the retry delay.
	pub fn WithRetryDelay(mut self, Delay:Duration) -> Self {
		self.RetryDelay = Delay;
		self
	}

	/// Enables or disables keepalive.
	pub fn WithKeepalive(mut self, Enabled:bool) -> Self {
		self.KeepaliveEnabled = Enabled;
		self
	}
}

/// Creates the default transport (gRPC to localhost).
pub fn CreateDefaultTransport() -> Strategy::Transport { Strategy::Transport::default() }

/// Creates a gRPC transport connecting to the given address.
pub fn CreategRPCTransport(Address:&str) -> Result<Strategy::Transport> {
	Ok(Strategy::Transport::gRPC(gRPCTransport::gRPCTransport::New(Address)?))
}

/// Creates an IPC transport using the default socket/pipe path.
pub fn CreateIPCTransport() -> Result<Strategy::Transport> {
	Ok(Strategy::Transport::IPC(IPCTransport::IPCTransport::New()?))
}

/// Creates a WASM transport with the given configuration.
pub fn CreateWASMTransport(
	EnableWASI:bool,
	MemoryLimitMegabytes:u64,
	MaxExecutionTimeMilliseconds:u64,
) -> Result<Strategy::Transport> {
	Ok(Strategy::Transport::WASM(WASMTransport::WASMTransportImpl::new(
		EnableWASI,
		MemoryLimitMegabytes,
		MaxExecutionTimeMilliseconds,
	)?))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn TestTransportConfigDefault() {
		let Configuration = TransportConfig::default();
		assert_eq!(
			Configuration.ConnectionTimeout.as_millis(),
			DEFAULT_CONNECTION_TIMEOUT_MS as u128
		);
	}

	#[test]
	fn TestTransportConfigBuilder() {
		let Configuration = TransportConfig::default()
			.WithConnectionTimeout(Duration::from_secs(10))
			.WithMaximumRetries(5);

		assert_eq!(Configuration.ConnectionTimeout.as_secs(), 10);
		assert_eq!(Configuration.MaximumRetries, 5);
	}

	#[test]
	fn TestTransportDefault() {
		let TransportValue = CreateDefaultTransport();
		match TransportValue {
			Strategy::Transport::gRPC(_) | Strategy::Transport::IPC(_) | Strategy::Transport::WASM(_) => {},
		}
	}
}
