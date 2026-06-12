//! Protocol configuration.

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
