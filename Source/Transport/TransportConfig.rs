//! Transport configuration struct and builder.

use std::time::Duration;

use super::{DEFAULT_CONNECTION_TIMEOUT_MS, DEFAULT_REQUEST_TIMEOUT_MS};

#[derive(Debug, Clone)]
pub struct TransportConfig {
	pub ConnectionTimeout:Duration,

	pub RequestTimeout:Duration,

	pub MaximumRetries:u32,

	pub RetryDelay:Duration,

	pub KeepaliveEnabled:bool,

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
	pub fn New() -> Self { Self::default() }

	pub fn WithConnectionTimeout(mut self, Timeout:Duration) -> Self {
		self.ConnectionTimeout = Timeout;

		self
	}

	pub fn WithRequestTimeout(mut self, Timeout:Duration) -> Self {
		self.RequestTimeout = Timeout;

		self
	}

	pub fn WithMaximumRetries(mut self, MaximumRetries:u32) -> Self {
		self.MaximumRetries = MaximumRetries;

		self
	}

	pub fn WithRetryDelay(mut self, Delay:Duration) -> Self {
		self.RetryDelay = Delay;

		self
	}

	pub fn WithKeepalive(mut self, Enabled:bool) -> Self {
		self.KeepaliveEnabled = Enabled;

		self
	}
}
