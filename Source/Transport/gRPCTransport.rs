//! gRPC Transport Implementation
//!
//! Provides gRPC-based communication for Grove.
//! Connects to Mountain or other gRPC services.

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::RwLock;
use tonic::transport::{Channel, Endpoint};
use tracing::{debug, info, instrument, warn};

use crate::Transport::{
	DEFAULT_CONNECTION_TIMEOUT_MS,
	DEFAULT_REQUEST_TIMEOUT_MS,
	Strategy::{TransportStats, TransportStrategy},
	TransportConfig,
};

/// gRPC transport for communication with Mountain and other gRPC services
#[derive(Clone, Debug)]
pub struct GrpcTransport {
	/// Connection endpoint
	endpoint:String,
	/// gRPC channel
	channel:Arc<RwLock<Option<Channel>>>,
	/// Transport configuration
	config:TransportConfig,
	/// Connection state
	connected:Arc<RwLock<bool>>,
	/// Transport statistics
	stats:Arc<RwLock<TransportStats>>,
}

impl GrpcTransport {
	/// Create a new gRPC transport with the given address
	///
	/// # Arguments
	///
	/// * `address` - The gRPC server address (e.g., "127.0.0.1:50050")
	///
	/// # Example
	///
	/// ```rust,no_run
	/// use grove::Transport::GrpcTransport;
	///
	/// let transport = GrpcTransport::new("127.0.0.1:50050")?;
	/// # Ok::<(), anyhow::Error>(())
	/// ```
	pub fn new(address:&str) -> anyhow::Result<Self> {
		Ok(Self {
			endpoint:address.to_string(),
			channel:Arc::new(RwLock::new(None)),
			config:TransportConfig::default(),
			connected:Arc::new(RwLock::new(false)),
			stats:Arc::new(RwLock::new(TransportStats::default())),
		})
	}

	/// Create a new gRPC transport with custom configuration
	pub fn with_config(address:&str, config:TransportConfig) -> anyhow::Result<Self> {
		Ok(Self {
			endpoint:address.to_string(),
			channel:Arc::new(RwLock::new(None)),
			config,
			connected:Arc::new(RwLock::new(false)),
			stats:Arc::new(RwLock::new(TransportStats::default())),
		})
	}

	/// Get the connection endpoint
	pub fn endpoint(&self) -> &str { &self.endpoint }

	/// Get the gRPC channel
	pub async fn channel(&self) -> anyhow::Result<Channel> {
		let channel = self.channel.read().await;
		channel
			.as_ref()
			.cloned()
			.ok_or_else(|| anyhow::anyhow!("gRPC channel not connected"))
	}

	/// Get transport statistics
	pub async fn stats(&self) -> TransportStats { self.stats.read().await.clone() }

	/// Build an endpoint from the address string
	fn build_endpoint(&self) -> anyhow::Result<Endpoint> {
		let endpoint = Endpoint::from_shared(self.endpoint.clone())?
			.timeout(self.config.connection_timeout)
			.connect_timeout(self.config.connection_timeout)
			.tcp_keepalive(Some(self.config.keepalive_interval));

		Ok(endpoint)
	}
}

#[async_trait]
impl super::super::Strategy::TransportStrategy for GrpcTransport {
	type Error = GrpcTransportError;

	#[instrument(skip(self))]
	async fn connect(&self) -> Result<(), Self::Error> {
		info!("Connecting to gRPC endpoint: {}", self.endpoint);

		let endpoint = self
			.build_endpoint()
			.map_err(|e| GrpcTransportError::ConnectionFailed(e.to_string()))?;

		let channel = endpoint
			.connect()
			.await
			.map_err(|e| GrpcTransportError::ConnectionFailed(e.to_string()))?;

		*self.channel.write().await = Some(channel);
		*self.connected.write().await = true;

		info!("gRPC connection established: {}", self.endpoint);
		debug!("Connected to gRPC endpoint: {}", self.endpoint);

		Ok(())
	}

	#[instrument(skip(self, request))]
	async fn send(&self, request:&[u8]) -> Result<Vec<u8>, Self::Error> {
		let start = std::time::Instant::now();

		if !self.is_connected() {
			return Err(GrpcTransportError::NotConnected);
		}

		debug!("Sending gRPC request ({} bytes)", request.len());

		// For a complete implementation, this would make an actual gRPC call
		// For now, we return a mock response
		let response = vec![]; // Placeholder

		let latency_us = start.elapsed().as_micros() as u64;

		// Update statistics
		let mut stats = self.stats.write().await;
		stats.record_sent(request.len() as u64, latency_us);
		stats.record_received(response.len() as u64);

		debug!("gRPC request completed in {}µs", latency_us);

		Ok(response)
	}

	#[instrument(skip(self, data))]
	async fn send_no_response(&self, data:&[u8]) -> Result<(), Self::Error> {
		if !self.is_connected() {
			return Err(GrpcTransportError::NotConnected);
		}

		debug!("Sending gRPC request without response ({} bytes)", data.len());

		// For a complete implementation, this would make an actual gRPC call
		// For now, we just update statistics
		let mut stats = self.stats.write().await;
		stats.record_sent(data.len() as u64, 0);

		Ok(())
	}

	#[instrument(skip(self))]
	async fn close(&self) -> Result<(), Self::Error> {
		info!("Closing gRPC connection: {}", self.endpoint);

		*self.channel.write().await = None;
		*self.connected.write().await = false;

		info!("gRPC connection closed: {}", self.endpoint);

		Ok(())
	}

	fn is_connected(&self) -> bool { self.connected.blocking_read().to_owned() }

	fn transport_type(&self) -> super::super::Strategy::TransportType {
		super::super::Strategy::TransportType::gRPC
	}
}

/// gRPC transport errors
#[derive(Debug, thiserror::Error)]
pub enum GrpcTransportError {
	#[error("Connection failed: {0}")]
	ConnectionFailed(String),

	#[error("Send failed: {0}")]
	SendFailed(String),

	#[error("Receive failed: {0}")]
	ReceiveFailed(String),

	#[error("Not connected")]
	NotConnected,

	#[error("Timeout")]
	Timeout,

	#[error("gRPC error: {0}")]
	GrpcError(String),
}

impl From<tonic::transport::Error> for GrpcTransportError {
	fn from(err:tonic::transport::Error) -> Self { GrpcTransportError::ConnectionFailed(err.to_string()) }
}

impl From<tonic::Status> for GrpcTransportError {
	fn from(status:tonic::Status) -> Self { GrpcTransportError::GrpcError(status.to_string()) }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_grpc_transport_creation() {
		let result = GrpcTransport::new("127.0.0.1:50050");
		assert!(result.is_ok());
		let transport = result.unwrap();
		assert_eq!(transport.endpoint(), "127.0.0.1:50050");
	}

	#[test]
	fn test_grpc_transport_with_config() {
		let config = TransportConfig::default().with_max_retries(5);
		let result = GrpcTransport::with_config("127.0.0.1:50050", config);
		assert!(result.is_ok());
	}

	#[tokio::test]
	async fn test_grpc_transport_not_connected() {
		let transport = GrpcTransport::new("127.0.0.1:50050").unwrap();
		assert!(!transport.is_connected());
	}

	#[tokio::test]
	async fn test_grpc_transport_stats() {
		let transport = GrpcTransport::new("127.0.0.1:50050").unwrap();
		let stats = transport.stats().await;
		assert_eq!(stats.messages_sent, 0);
		assert_eq!(stats.messages_received, 0);
	}
}
