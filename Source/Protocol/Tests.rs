//! Protocol tests.

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
