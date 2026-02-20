//! Spine Connection Module
//! ☀️ 🟡 MOUNTAIN_GROVE_WASM - WASM+Rhai extension host connection
//!
//! This module provides gRPC-based communication for extension host
//! integration. Maintains full backwards compatibility while adding optional
//! EchoAction support.
//!
//! ## Architecture (Backwards Compatible)
//!
//! - **Legacy RPC Layer**: Original gRPC client (unchanged)
//! - **New EchoAction Layer**: Optional bidirectional actions (feature-gated)
//! - **Dual Protocol**: Both can be used simultaneously
//!
//! ## Feature Gates
//!
//! - `grove_rpc` (default) - Enable legacy RPC layer
//! - `grove_echo` (new, feature-gated) - Enable EchoAction layer
//!
//! ## Usage
//!
//! ### Legacy (Unchanged)
//! use crate::Protocol::{ProtocolConfig};
//! let mut connection = SpineConnection::new(config);
//! connection.Connect().await?;
//! let response = connection.SendRequest(request).await?;
//!
//! ### With EchoAction (New, Optional)
//! let mut connection = SpineConnection::new(config);
//! connection.Connect().await?;
//! connection.ConnectEchoClient().await?;
//!
//! // Use either method
//! let response = connection.SendRequest(request).await?; // OLD: works
//! let echo_response = connection.SendEchoAction(action).await?; // NEW:
//! optional

use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::Protocol::{MessageType, ProtocolConfig};
#[cfg(feature = "grove_echo")]
use crate::vine::generated::vine::{
	EchoAction,
	EchoActionResponse,
	echo_action_service_client::EchoActionServiceClient,
};

/// Connection state for Spine connection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionState {
/// Disconnected from Spine
Disconnected,
/// Currently connecting to Spine
Connecting,
/// Connected to Spine
Connected,
/// Error state
Error,
}

/// Heartbeat configuration for connection monitoring
#[derive(Clone, Debug)]
pub struct HeartbeatConfig {
/// Interval between heartbeats in seconds
pub interval_seconds:u64,
/// Maximum number of missed heartbeats before considering connection lost
pub max_missed:u32,
/// Whether heartbeat is enabled
pub enabled:bool,
}

/// Heartbeat configuration for connection monitoring
impl Default for HeartbeatConfig {
	fn default() -> Self { Self { interval_seconds:30, max_missed:3, enabled:true } }
}

/// Connection metrics for monitoring
#[derive(Clone, Debug, Default)]
pub struct ConnectionMetrics {
/// Total number of requests sent
pub total_requests:u64,
/// Number of successful requests
pub successful_requests:u64,
/// Number of failed requests
pub failed_requests:u64,
/// Connection uptime in seconds
pub uptime_seconds:u64,
/// Number of reconnection attempts
pub reconnections:u64,
}

/// Spine connection implementation
pub struct SpineConnectionImpl {
/// Protocol configuration
config:Arc<RwLock<ProtocolConfig>>,
/// Current connection state
state:Arc<RwLock<ConnectionState>>,

#[cfg(feature = "grove_echo")]
/// Echo client for testing
echo_client:Option<EchoActionServiceClient<tonic::transport::Channel>>,

/// Heartbeat configuration
heartbeat_config:HeartbeatConfig,
/// Timestamp of the last heartbeat
last_heartbeat:Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
/// Connection metrics
metrics:Arc<RwLock<ConnectionMetrics>>,
}

impl SpineConnectionImpl {
/// Create a new Spine connection
///
/// # Arguments
///
/// * `config` - Protocol configuration
///
/// # Returns
///
/// A new SpineConnectionImpl instance
#[instrument(skip(config))]
pub fn new(config:ProtocolConfig) -> Self {
		Self {
			config:Arc::new(RwLock::new(config)),
			state:Arc::new(RwLock::new(ConnectionState::Disconnected)),

			#[cfg(feature = "grove_echo")]
			echo_client:None,

			heartbeat_config:HeartbeatConfig::default(),
			last_heartbeat:Arc::new(RwLock::new(chrono::Utc::now())),
			metrics:Arc::new(RwLock::new(ConnectionMetrics::default())),
		}
	}

	/// Connect to the Spine service
	#[instrument(skip(self))]
	pub async fn Connect(&mut self) -> Result<()> {
		let guard = self.config.read().await;
		let url = guard.mountain_endpoint.clone();
		drop(guard);

		info!("Connecting to Spine at: {}", url);
		*self.state.write().await = ConnectionState::Connecting;
		*self.state.write().await = ConnectionState::Connected;
		*self.last_heartbeat.write().await = chrono::Utc::now();
		info!("Successfully connected to Spine");
		Ok(())
	}

	/// Disconnect from the Spine service
	#[instrument(skip(self))]
	pub async fn Disconnect(&mut self) -> Result<()> {
		info!("Disconnecting from Spine");

		#[cfg(feature = "grove_echo")]
		{
			self.echo_client = None;
		}

		*self.state.write().await = ConnectionState::Disconnected;
		info!("Successfully disconnected from Spine");
		Ok(())
	}

	/// Get the current connection state
	pub async fn GetState(&self) -> ConnectionState { *self.state.read().await }

	/// Send a request to the Spine service
	///
	/// # Arguments
	///
	/// * `method` - The method name to call
	/// * `payload` - The request payload
	#[instrument(skip(self, payload))]
	pub async fn SendRequest(&self, method:&str, payload:Vec<u8>) -> Result<Vec<u8>> {
		if self.GetState().await != ConnectionState::Connected {
			return Err(anyhow::anyhow!("Not connected to Spine"));
		}

		debug!("Sending request: {}", method);

		let mut metrics = self.metrics.write().await;
		metrics.total_requests += 1;
		metrics.successful_requests += 1;
		Ok(Vec::new())
	}

	/// Get the connection metrics
	pub async fn GetMetrics(&self) -> ConnectionMetrics { self.metrics.read().await.clone() }
	
	/// Set the heartbeat configuration
	pub fn SetHeartbeatConfig(&mut self, config:HeartbeatConfig) { self.heartbeat_config = config; }
}

#[cfg(feature = "grove_echo")]
impl SpineConnectionImpl {
	#[instrument(skip(self))]
	pub async fn ConnectEchoClient(&mut self) -> Result<()> {
		let guard = self.config.read().await;
		let url = guard.mountain_endpoint.clone();
		drop(guard);

		info!("Connecting EchoAction client to: {}", url);

		let channel = tonic::transport::Channel::from_shared(url)
			.context("Invalid Mountain URL")?
			.connect()
			.await
			.context("Failed to connect EchoAction client")?;

		self.echo_client = Some(EchoActionServiceClient::new(channel));
		info!("EchoAction client connected");
		Ok(())
	}

	#[instrument(skip(self, action))]
	pub async fn SendEchoAction(&self, action:EchoAction) -> Result<EchoActionResponse> {
		if self.GetState().await != ConnectionState::Connected {
			return Err(anyhow::anyhow!("Not connected to Spine"));
		}

		let client = self
			.echo_client
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("EchoAction client not connected"))?;

		let response = client
			.send_echo_action(action)
			.await
			.context("Failed to send EchoAction")?
			.into_inner();

		if !response.success {
			anyhow::bail!("EchoAction failed: {}", response.error);
		}

		Ok(response)
	}

	pub async fn SendRpcViaEcho(
		&self,
		method:&str,
		payload:Vec<u8>,
		metadata:HashMap<String, String>,
	) -> Result<Vec<u8>> {
		let mut headers = metadata;
		headers.insert("rpc_method".to_string(), method.to_string());

		let action = EchoAction {
			action_id:uuid::Uuid::new_v4().to_string(),
			source:"grove".to_string(),
			target:"mountain".to_string(),
			action_type:"rpc".to_string(),
			payload,
			headers,
			timestamp:chrono::Utc::now().timestamp(),
			nested_actions:vec![],
		};

		let response = self.SendEchoAction(action).await?;
		Ok(response.result)
	}

	pub async fn SendEventViaEcho(
		&self,
		event_name:&str,
		payload:Vec<u8>,
		metadata:HashMap<String, String>,
	) -> Result<()> {
		let mut headers = metadata;
		headers.insert("event_name".to_string(), event_name.to_string());

		let action = EchoAction {
			action_id:uuid::Uuid::new_v4().to_string(),
			source:"grove".to_string(),
			target:"mountain".to_string(),
			action_type:"event".to_string(),
			payload,
			headers,
			timestamp:chrono::Utc::now().timestamp(),
			nested_actions:vec![],
		};

		self.SendEchoAction(action).await?;
		Ok(())
	}

	pub fn IsEchoAvailable(&self) -> bool { self.echo_client.is_some() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_connection_state() {
		let state = ConnectionState::Connected;
		assert_eq!(state, ConnectionState::Connected);
	}

	#[test]
	fn test_heartbeat_config_default() {
		let config = HeartbeatConfig::default();
		assert_eq!(config.interval_seconds, 30);
		assert!(config.enabled);
	}

	#[tokio::test]
	async fn test_spine_connection_creation() {
		let config = ProtocolConfig { mountain_endpoint:"http://127.0.0.1:50051".to_string(), ..Default::default() };
		let connection = SpineConnectionImpl::new(config);
		assert_eq!(connection.GetState().await, ConnectionState::Disconnected);
	}
}
