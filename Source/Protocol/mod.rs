//! Protocol Module
//!
//! Handles protocol communication with Mountain and other services.
//! Implements the Spine protocol for extension host communication.

pub mod SpineConnection;

// Re-exports for convenience - use module prefix to avoid E0255 conflicts
// Note: SpineConnection must be accessed via SpineConnection::SpineConnectionImpl

/// Protocol version
pub const SPINE_PROTOCOL_VERSION:&str = "1.0.0";

/// Default Mountain gRPC endpoint
pub const DEFAULT_MOUNTAIN_ENDPOINT:&str = "127.0.0.1:50050";

/// Default connection timeout in milliseconds
pub const DEFAULT_CONNECTION_TIMEOUT_MS:u64 = 5000;

/// Default heartbeat interval in seconds
pub const DEFAULT_HEARTBEAT_INTERVAL_SEC:u64 = 30;

/// Default message buffer size
pub const DEFAULT_MESSAGE_BUFFER_SIZE:usize = 8192;

/// Protocol configuration
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
	/// Protocol version
	pub version:String,
	/// Mountain endpoint
	pub mountain_endpoint:String,
	/// Connection timeout
	pub connection_timeout_ms:u64,
	/// Heartbeat interval
	pub heartbeat_interval_sec:u64,
	/// Message buffer size
	pub message_buffer_size:usize,
	/// Enable TLS
	pub enable_tls:bool,
	/// Enable compression
	pub enable_compression:bool,
}

impl ProtocolConfig {
	/// Create a new protocol configuration
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

	/// Set mountain endpoint
	pub fn with_mountain_endpoint(mut self, endpoint:String) -> Self {
		self.mountain_endpoint = endpoint;
		self
	}

	/// Set connection timeout
	pub fn with_connection_timeout(mut self, timeout_ms:u64) -> Self {
		self.connection_timeout_ms = timeout_ms;
		self
	}

	/// Set heartbeat interval
	pub fn with_heartbeat_interval(mut self, interval_sec:u64) -> Self {
		self.heartbeat_interval_sec = interval_sec;
		self
	}

	/// Enable or disable TLS
	pub fn with_tls(mut self, enable:bool) -> Self {
		self.enable_tls = enable;
		self
	}

	/// Enable or disable compression
	pub fn with_compression(mut self, enable:bool) -> Self {
		self.enable_compression = enable;
		self
	}
}

impl Default for ProtocolConfig {
	fn default() -> Self { Self::new() }
}

/// Message types for Spine protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageType {
	/// Heartbeat message
	Heartbeat = 0,
	/// Registration message
	Register = 1,
	/// Unregistration message
	Unregister = 2,
	/// Event message
	Event = 3,
	/// Request message
	Request = 4,
	/// Response message
	Response = 5,
	/// Error message
	Error = 6,
}

impl MessageType {
	/// Convert to u32
	pub fn as_u32(self) -> u32 { self as u32 }

	/// Convert from u32
	pub fn from_u32(value:u32) -> Option<Self> {
		match value {
			0 => Some(Self::Heartbeat),
			1 => Some(Self::Register),
			2 => Some(Self::Unregister),
			3 => Some(Self::Event),
			4 => Some(Self::Request),
			5 => Some(Self::Response),
			6 => Some(Self::Error),
			_ => None,
		}
	}
}

/// Protocol error types
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
/// Connection error
#[error("Connection error: {0}")]
ConnectionError(String),

/// Serialization error
#[error("Serialization error: {0}")]
SerializationError(String),

/// Deserialization error
#[error("Deserialization error: {0}")]
DeserializationError(String),

/// Invalid message error
#[error("Invalid message: {0}")]
InvalidMessage(String),

/// Timeout error
#[error("Timeout error")]
Timeout,

/// Protocol error
#[error("Protocol error: {0}")]
ProtocolError(String),
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_protocol_config_default() {
		let config = ProtocolConfig::default();
		assert_eq!(config.mountain_endpoint, DEFAULT_MOUNTAIN_ENDPOINT);
		assert_eq!(config.connection_timeout_ms, DEFAULT_CONNECTION_TIMEOUT_MS);
	}

	#[test]
	fn test_protocol_config_builder() {
		let config = ProtocolConfig::default()
			.with_mountain_endpoint("127.0.0.1:60000".to_string())
			.with_connection_timeout(10000)
			.with_heartbeat_interval(60);

		assert_eq!(config.mountain_endpoint, "127.0.0.1:60000");
		assert_eq!(config.connection_timeout_ms, 10000);
		assert_eq!(config.heartbeat_interval_sec, 60);
	}

	#[test]
	fn test_message_type_conversion() {
		let msg_type = MessageType::Heartbeat;
		assert_eq!(msg_type.as_u32(), 0);

		let converted = MessageType::from_u32(0);
		assert_eq!(converted, Some(MessageType::Heartbeat));

		let invalid = MessageType::from_u32(999);
		assert_eq!(invalid, None);
	}
}
