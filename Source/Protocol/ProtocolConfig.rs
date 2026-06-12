//! Protocol configuration struct and builder.

use super::{
	DEFAULT_CONNECTION_TIMEOUT_MS,
	DEFAULT_HEARTBEAT_INTERVAL_SEC,
	DEFAULT_MESSAGE_BUFFER_SIZE,
	DEFAULT_MOUNTAIN_ENDPOINT,
	SPINE_PROTOCOL_VERSION,
};

#[derive(Debug, Clone)]
pub struct ProtocolConfig {
	pub version:String,

	pub mountain_endpoint:String,

	pub connection_timeout_ms:u64,

	pub heartbeat_interval_sec:u64,

	pub message_buffer_size:usize,

	pub enable_tls:bool,

	pub enable_compression:bool,
}

impl ProtocolConfig {
	pub fn new() -> Self {
		Self {
			version:SPINE_PROTOCOL_VERSION.to_string(),

			mountain_endpoint:DEFAULT_MOUNTAIN_ENDPOINT.to_string(),

			connection_timeout_ms:DEFAULT_CONNECTION_TIMEOUT_MS,

			heartbeat_interval_sec:DEFAULT_HEARTBEAT_INTERVAL_SEC,

			message_buffer_size:DEFAULT_MESSAGE_BUFFER_SIZE,

			enable_tls:false,

			enable_compression:false,
		}
	}

	pub fn with_mountain_endpoint(mut self, endpoint:String) -> Self {
		self.mountain_endpoint = endpoint;

		self
	}

	pub fn with_connection_timeout(mut self, timeout_ms:u64) -> Self {
		self.connection_timeout_ms = timeout_ms;

		self
	}

	pub fn with_heartbeat_interval(mut self, interval_sec:u64) -> Self {
		self.heartbeat_interval_sec = interval_sec;

		self
	}

	pub fn with_tls(mut self, enable:bool) -> Self {
		self.enable_tls = enable;

		self
	}

	pub fn with_compression(mut self, enable:bool) -> Self {
		self.enable_compression = enable;

		self
	}
}

impl Default for ProtocolConfig {
	fn default() -> Self { Self::new() }
}
