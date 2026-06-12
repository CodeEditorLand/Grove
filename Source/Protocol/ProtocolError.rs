//! Protocol errors.

use thiserror::Error;

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
