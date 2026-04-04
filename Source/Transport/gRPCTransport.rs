#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
//! # gRPC Transport Implementation
//!
//! Provides gRPC-based communication for Grove.
//! Connects to Mountain or other gRPC services.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tonic::transport::{Channel, Endpoint};
use tracing::{debug, info, instrument};

use crate::Transport::{
	Strategy::{TransportStats, TransportStrategy, TransportType},
	TransportConfig,
};

/// gRPC transport for communication with Mountain and other gRPC services.
#[derive(Clone, Debug)]
pub struct gRPCTransport {
	/// Connection endpoint address.
	Endpoint: String,
	/// gRPC channel (lazily connected).
	Channel: Arc<RwLock<Option<Channel>>>,
	/// Transport configuration.
	Configuration: TransportConfig,
	/// Whether the transport is currently connected.
	Connected: Arc<RwLock<bool>>,
	/// Transport statistics.
	Statistics: Arc<RwLock<TransportStats>>,
}

impl gRPCTransport {
	/// Creates a new gRPC transport with the given address.
	pub fn New(Address: &str) -> anyhow::Result<Self> {
		Ok(Self {
			Endpoint: Address.to_string(),
			Channel: Arc::new(RwLock::new(None)),
			Configuration: TransportConfig::default(),
			Connected: Arc::new(RwLock::new(false)),
			Statistics: Arc::new(RwLock::new(TransportStats::default())),
		})
	}

	/// Creates a new gRPC transport with custom configuration.
	pub fn WithConfiguration(
		Address: &str,
		Configuration: TransportConfig,
	) -> anyhow::Result<Self> {
		Ok(Self {
			Endpoint: Address.to_string(),
			Channel: Arc::new(RwLock::new(None)),
			Configuration,
			Connected: Arc::new(RwLock::new(false)),
			Statistics: Arc::new(RwLock::new(TransportStats::default())),
		})
	}

	/// Returns the connection endpoint address.
	pub fn Address(&self) -> &str { &self.Endpoint }

	/// Returns the active gRPC channel.
	pub async fn GetChannel(&self) -> anyhow::Result<Channel> {
		self.Channel
			.read()
			.await
			.as_ref()
			.cloned()
			.ok_or_else(|| anyhow::anyhow!("gRPC channel not connected"))
	}

	/// Returns a snapshot of transport statistics.
	pub async fn Statistics(&self) -> TransportStats { self.Statistics.read().await.clone() }

	/// Builds an endpoint from the address string.
	fn BuildEndpoint(&self) -> anyhow::Result<Endpoint> {
		let EndpointValue = Endpoint::from_shared(self.Endpoint.clone())?
			.timeout(self.Configuration.ConnectionTimeout)
			.connect_timeout(self.Configuration.ConnectionTimeout)
			.tcp_keepalive(Some(self.Configuration.KeepaliveInterval));
		Ok(EndpointValue)
	}
}

#[async_trait]
impl TransportStrategy for gRPCTransport {
	type Error = gRPCTransportError;

	#[instrument(skip(self))]
	async fn connect(&self) -> Result<(), Self::Error> {
		info!("Connecting to gRPC endpoint: {}", self.Endpoint);

		let EndpointValue = self
			.BuildEndpoint()
			.map_err(|E| gRPCTransportError::ConnectionFailed(E.to_string()))?;

		let ChannelValue = EndpointValue
			.connect()
			.await
			.map_err(|E| gRPCTransportError::ConnectionFailed(E.to_string()))?;

		*self.Channel.write().await = Some(ChannelValue);
		*self.Connected.write().await = true;

		info!("gRPC connection established: {}", self.Endpoint);
		Ok(())
	}

	#[instrument(skip(self, request))]
	async fn send(&self, request: &[u8]) -> Result<Vec<u8>, Self::Error> {
		let Start = std::time::Instant::now();

		if !self.is_connected() {
			return Err(gRPCTransportError::NotConnected);
		}

		debug!("Sending gRPC request ({} bytes)", request.len());

		let Response: Vec<u8> = vec![];
		let LatencyMicroseconds = Start.elapsed().as_micros() as u64;

		let mut Stats = self.Statistics.write().await;
		Stats.record_sent(request.len() as u64, LatencyMicroseconds);
		Stats.record_received(Response.len() as u64);

		debug!("gRPC request completed in {}µs", LatencyMicroseconds);
		Ok(Response)
	}

	#[instrument(skip(self, data))]
	async fn send_no_response(&self, data: &[u8]) -> Result<(), Self::Error> {
		if !self.is_connected() {
			return Err(gRPCTransportError::NotConnected);
		}

		debug!("Sending gRPC notification ({} bytes)", data.len());

		let mut Stats = self.Statistics.write().await;
		Stats.record_sent(data.len() as u64, 0);
		Ok(())
	}

	#[instrument(skip(self))]
	async fn close(&self) -> Result<(), Self::Error> {
		info!("Closing gRPC connection: {}", self.Endpoint);
		*self.Channel.write().await = None;
		*self.Connected.write().await = false;
		info!("gRPC connection closed: {}", self.Endpoint);
		Ok(())
	}

	fn is_connected(&self) -> bool { *self.Connected.blocking_read() }

	fn transport_type(&self) -> TransportType { TransportType::gRPC }
}

/// gRPC transport error variants.
#[derive(Debug, thiserror::Error)]
pub enum gRPCTransportError {
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
	Error(String),
}

impl From<tonic::transport::Error> for gRPCTransportError {
	fn from(Error: tonic::transport::Error) -> Self {
		gRPCTransportError::ConnectionFailed(Error.to_string())
	}
}

impl From<tonic::Status> for gRPCTransportError {
	fn from(Status: tonic::Status) -> Self { gRPCTransportError::Error(Status.to_string()) }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn TestgRPCTransportCreation() {
		let Result = gRPCTransport::New("127.0.0.1:50050");
		assert!(Result.is_ok());
		let Transport = Result.unwrap();
		assert_eq!(Transport.Address(), "127.0.0.1:50050");
	}

	#[tokio::test]
	async fn TestgRPCTransportNotConnected() {
		let Transport = gRPCTransport::New("127.0.0.1:50050").unwrap();
		assert!(!Transport.is_connected());
	}
}
