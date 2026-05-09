//! Transport Strategy Module
//!
//! Defines the transport strategy trait and types for different
//! communication methods (gRPC, IPC, WASM).

use std::fmt;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::Transport::{IPCTransport::IPCTransport, WASMTransport::WASMTransportImpl, gRPCTransport::gRPCTransport};

/// Transport strategy trait
///
/// All transport implementations must implement this trait to provide
/// a common interface for connecting, sending, and closing connections.
#[async_trait]
pub trait TransportStrategy: Send + Sync {
	/// Error type for this transport
	type Error: std::error::Error + Send + Sync + 'static;

	/// Connect to the transport endpoint
	async fn connect(&self) -> Result<(), Self::Error>;

	/// Send a request and receive a response
	async fn send(&self, request:&[u8]) -> Result<Vec<u8>, Self::Error>;

	/// Send data without expecting a response (fire and forget)
	async fn send_no_response(&self, data:&[u8]) -> Result<(), Self::Error>;

	/// Close the transport connection
	async fn close(&self) -> Result<(), Self::Error>;

	/// Check if the transport is connected
	fn is_connected(&self) -> bool;

	/// Get the transport type identifier
	fn transport_type(&self) -> TransportType;
}

/// Transport type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportType {
	/// gRPC transport
	gRPC,

	/// Inter-process communication
	IPC,

	/// Direct WASM module communication
	WASM,

	/// Unknown/unspecified transport
	Unknown,
}

impl fmt::Display for TransportType {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::gRPC => write!(f, "grpc"),

			Self::IPC => write!(f, "ipc"),

			Self::WASM => write!(f, "wasm"),

			Self::Unknown => write!(f, "unknown"),
		}
	}
}

impl std::str::FromStr for TransportType {
	type Err = anyhow::Error;

	fn from_str(s:&str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"grpc" => Ok(Self::gRPC),

			"ipc" => Ok(Self::IPC),

			"wasm" => Ok(Self::WASM),

			_ => Err(anyhow::anyhow!("Unknown transport type: {}", s)),
		}
	}
}

/// Transport enumeration.
///
/// Union type wrapping all supported transport implementations.
#[derive(Debug)]
pub enum Transport {
	/// gRPC-based transport (Mountain/Air communication).
	gRPC(gRPCTransport),

	/// IPC transport (same-machine process communication).
	IPC(IPCTransport),

	/// Direct WASM module transport (browser).
	WASM(WASMTransportImpl),
}

impl Transport {
	/// Get the transport type
	pub fn transport_type(&self) -> TransportType {
		match self {
			Self::gRPC(_) => TransportType::gRPC,

			Self::IPC(_) => TransportType::IPC,

			Self::WASM(_) => TransportType::WASM,
		}
	}

	/// Connect to the transport
	pub async fn connect(&self) -> anyhow::Result<()> {
		match self {
			Self::gRPC(transport) => {
				transport
					.connect()
					.await
					.map_err(|e| anyhow::anyhow!("gRPC connect error: {}", e))
			},

			Self::IPC(transport) => {
				transport
					.connect()
					.await
					.map_err(|e| anyhow::anyhow!("IPC connect error: {}", e))
			},

			Self::WASM(transport) => {
				transport
					.connect()
					.await
					.map_err(|e| anyhow::anyhow!("WASM connect error: {}", e))
			},
		}
	}

	/// Send a request and receive a response
	pub async fn send(&self, request:&[u8]) -> anyhow::Result<Vec<u8>> {
		match self {
			Self::gRPC(transport) => {
				transport
					.send(request)
					.await
					.map_err(|e| anyhow::anyhow!("gRPC send error: {}", e))
			},

			Self::IPC(transport) => {
				transport
					.send(request)
					.await
					.map_err(|e| anyhow::anyhow!("IPC send error: {}", e))
			},

			Self::WASM(transport) => {
				transport
					.send(request)
					.await
					.map_err(|e| anyhow::anyhow!("WASM send error: {}", e))
			},
		}
	}

	/// Send data without expecting a response
	pub async fn send_no_response(&self, data:&[u8]) -> anyhow::Result<()> {
		match self {
			Self::gRPC(transport) => {
				transport
					.send_no_response(data)
					.await
					.map_err(|e| anyhow::anyhow!("gRPC send error: {}", e))
			},

			Self::IPC(transport) => {
				transport
					.send_no_response(data)
					.await
					.map_err(|e| anyhow::anyhow!("IPC send error: {}", e))
			},

			Self::WASM(transport) => {
				transport
					.send_no_response(data)
					.await
					.map_err(|e| anyhow::anyhow!("WASM send error: {}", e))
			},
		}
	}

	/// Close the transport
	pub async fn close(&self) -> anyhow::Result<()> {
		match self {
			Self::gRPC(transport) => transport.close().await.map_err(|e| anyhow::anyhow!("gRPC close error: {}", e)),

			Self::IPC(transport) => transport.close().await.map_err(|e| anyhow::anyhow!("IPC close error: {}", e)),

			Self::WASM(transport) => transport.close().await.map_err(|e| anyhow::anyhow!("WASM close error: {}", e)),
		}
	}

	/// Check if the transport is connected
	pub fn is_connected(&self) -> bool {
		match self {
			Self::gRPC(transport) => transport.is_connected(),

			Self::IPC(transport) => transport.is_connected(),

			Self::WASM(transport) => transport.is_connected(),
		}
	}

	/// Get gRPC transport reference (if applicable)
	pub fn AsgRPC(&self) -> Option<&gRPCTransport> {
		match self {
			Self::gRPC(Transport) => Some(Transport),

			_ => None,
		}
	}

	/// Returns the IPC transport reference if this is an IPC transport.
	pub fn AsIPC(&self) -> Option<&IPCTransport> {
		match self {
			Self::IPC(Transport) => Some(Transport),

			_ => None,
		}
	}

	/// Get WASM transport reference (if applicable)
	pub fn as_wasm(&self) -> Option<&WASMTransportImpl> {
		match self {
			Self::WASM(transport) => Some(transport),

			_ => None,
		}
	}
}

impl Default for Transport {
	fn default() -> Self {
		Self::gRPC(
			gRPCTransport::New("127.0.0.1:50050").unwrap_or_else(|_| {
				gRPCTransport::New("0.0.0.0:50050").expect("Failed to create default gRPC transport")
			}),
		)
	}
}

impl fmt::Display for Transport {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Transport({})", self.transport_type()) }
}

/// Transport message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMessage {
	/// Message type identifier
	pub message_type:String,

	/// Message ID for correlation
	pub message_id:String,

	/// Timestamp (Unix epoch)
	pub timestamp:u64,

	/// Message payload
	pub payload:Bytes,

	/// Optional metadata
	pub metadata:Option<serde_json::Value>,
}

impl TransportMessage {
	/// Create a new transport message
	pub fn new(message_type:impl Into<String>, payload:Bytes) -> Self {
		Self {
			message_type:message_type.into(),

			message_id:uuid::Uuid::new_v4().to_string(),

			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_secs())
				.unwrap_or(0),

			payload,

			metadata:None,
		}
	}

	/// Set metadata for the message
	pub fn with_metadata(mut self, metadata:serde_json::Value) -> Self {
		self.metadata = Some(metadata);

		self
	}

	/// Serialize the message to bytes
	pub fn to_bytes(&self) -> anyhow::Result<Bytes> {
		serde_json::to_vec(self).map(Bytes::from).map_err(|e| anyhow::anyhow!("{}", e))
	}

	/// Deserialize message from bytes
	pub fn from_bytes(bytes:&[u8]) -> anyhow::Result<Self> {
		serde_json::from_slice(bytes).map_err(|e| anyhow::anyhow!("{}", e))
	}
}

/// Transport statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransportStats {
	/// Number of messages sent
	pub messages_sent:u64,

	/// Number of messages received
	pub messages_received:u64,

	/// Number of errors encountered
	pub errors:u64,

	/// Total bytes sent
	pub bytes_sent:u64,

	/// Total bytes received
	pub bytes_received:u64,

	/// Average latency in microseconds
	pub avg_latency_us:u64,

	/// Connection uptime in seconds
	pub uptime_seconds:u64,
}

impl TransportStats {
	/// Update statistics with a sent message
	pub fn record_sent(&mut self, bytes:u64, latency_us:u64) {
		self.messages_sent += 1;

		self.bytes_sent += bytes;

		// Update average latency
		if self.messages_sent > 0 {
			self.avg_latency_us = (self.avg_latency_us * (self.messages_sent - 1) + latency_us) / self.messages_sent;
		}
	}

	/// Update statistics with a received message
	pub fn record_received(&mut self, bytes:u64) {
		self.messages_received += 1;

		self.bytes_received += bytes;
	}

	/// Record an error
	pub fn record_error(&mut self) { self.errors += 1; }
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_transport_type_to_string() {
		assert_eq!(TransportType::gRPC.to_string(), "grpc");

		assert_eq!(TransportType::IPC.to_string(), "ipc");

		assert_eq!(TransportType::WASM.to_string(), "wasm");
	}

	#[test]
	fn test_transport_type_from_str() {
		assert_eq!("grpc".parse::<TransportType>().unwrap(), TransportType::gRPC);

		assert_eq!("ipc".parse::<TransportType>().unwrap(), TransportType::IPC);

		assert_eq!("wasm".parse::<TransportType>().unwrap(), TransportType::WASM);

		assert!("unknown".parse::<TransportType>().is_err());
	}

	#[test]
	fn test_transport_display() {
		// Create a dummy transport to test Display implementation
		// In real tests, we'd use an actual transport
		let transport = Transport::default();

		let display = format!("{}", transport);

		assert!(display.contains("Transport"));
	}

	#[test]
	fn test_transport_message_creation() {
		let message = TransportMessage::new("test_type", Bytes::from("hello"));

		assert_eq!(message.message_type, "test_type");

		assert_eq!(message.payload, Bytes::from("hello"));

		assert!(!message.message_id.is_empty());
	}

	#[test]
	fn test_transport_message_serialization() {
		let message = TransportMessage::new("test", Bytes::from("data"));

		let bytes = message.to_bytes().unwrap();

		let deserialized = TransportMessage::from_bytes(&bytes).unwrap();

		assert_eq!(deserialized.message_type, message.message_type);

		assert_eq!(deserialized.payload, message.payload);
	}

	#[test]
	fn test_transport_stats() {
		let mut stats = TransportStats::default();

		stats.record_sent(100, 1000);

		stats.record_received(50);

		stats.record_error();

		assert_eq!(stats.messages_sent, 1);

		assert_eq!(stats.messages_received, 1);

		assert_eq!(stats.errors, 1);

		assert_eq!(stats.bytes_sent, 100);

		assert_eq!(stats.bytes_received, 50);

		assert_eq!(stats.avg_latency_us, 1000);
	}
}
