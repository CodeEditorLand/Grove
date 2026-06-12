//! Protocol error types.

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
	#[error("Connection error: {0}")]
	ConnectionError(String),

	#[error("Serialization error: {0}")]
	SerializationError(String),

	#[error("Deserialization error: {0}")]
	DeserializationError(String),

	#[error("Invalid message: {0}")]
	InvalidMessage(String),

	#[error("Timeout error")]
	Timeout,

	#[error("Protocol error: {0}")]
	ProtocolError(String),
}
