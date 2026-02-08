//! Spine Connection Module
//!
//! Handles the Spine protocol connection with Mountain.
//! Provides gRPC-based communication for extension host integration.

use crate::Protocol::{ProtocolConfig, MessageType, ProtocolError};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Spine connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is disconnected
    Disconnected,
    /// Connection is connecting
    Connecting,
    /// Connection is connected
    Connected,
    /// Connection is reconnecting
    Reconnecting,
    /// Connection encountered an error
    Error,
}

/// Spine configuration
#[derive(Debug, Clone)]
pub struct SpineConfig {
    /// Protocol configuration
    pub protocol: ProtocolConfig,
    /// Service name
    pub service_name: String,
    /// Enable auto-reconnect
    pub auto_reconnect: bool,
    /// Maximum reconnect attempts
    pub max_reconnect_attempts: u32,
    /// Reconnect delay in milliseconds
    pub reconnect_delay_ms: u64,
}

impl SpineConfig {
    /// Create a new Spine configuration
    pub fn new(service_name: String) -> Self {
        Self {
            protocol: ProtocolConfig::default(),
            service_name,
            auto_reconnect: true,
            max_reconnect_attempts: 3,
            reconnect_delay_ms: 5000,
        }
    }

    /// Set protocol configuration
    pub fn with_protocol(mut self, protocol: ProtocolConfig) -> Self {
        self.protocol = protocol;
        self
    }

    /// Set auto-reconnect
    pub fn with_auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }

    /// Set maximum reconnect attempts
    pub fn with_max_reconnect_attempts(mut self, max: u32) -> Self {
        self.max_reconnect_attempts = max;
        self
    }
}

/// Spine message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpineMessage {
    /// Message type
    pub message_type: MessageType,
    /// Message ID for correlation
    pub message_id: String,
    /// Message timestamp
    pub timestamp: u64,
    /// Message payload
    pub payload: serde_json::Value,
    /// Optional correlation ID
    pub correlation_id: Option<String>,
}

impl SpineMessage {
    /// Create a new Spine message
    pub fn new(message_type: MessageType) -> Self {
        Self {
            message_type,
            message_id: uuid::Uuid::new_v4().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            payload: serde_json::Value::Null,
            correlation_id: None,
        }
    }

    /// Create a request message
    pub fn request(payload: serde_json::Value) -> Self {
        let mut msg = Self::new(MessageType::Request);
        msg.payload = payload;
        msg
    }

    /// Create a response message
    pub fn response(payload: serde_json::Value, correlation_id: String) -> Self {
        let mut msg = Self::new(MessageType::Response);
        msg.payload = payload;
        msg.correlation_id = Some(correlation_id);
        msg
    }

    /// Create an error message
    pub fn error(error: String, correlation_id: Option<String>) -> Self {
        let mut msg = Self::new(MessageType::Error);
        msg.payload = serde_json::json!({ "error": error });
        msg.correlation_id = correlation_id;
        msg
    }

    /// Create a heartbeat message
    pub fn heartbeat() -> Self {
        Self::new(MessageType::Heartbeat)
    }
}

/// Spine connection handler
pub struct SpineConnection {
    /// Configuration
    config: SpineConfig,
    /// Connection state
    state: Arc<RwLock<ConnectionState>>,
    /// Connected flag
    connected: Arc<RwLock<bool>>,
    /// Message callbacks
    message_callbacks: Arc<RwLock<Vec<MessageCallback>>>,
    /// Statistics
    stats: Arc<RwLock<SpineStats>>,
}

/// Spine connection statistics
#[derive(Debug, Clone, Default)]
pub struct SpineStats {
    /// Messages sent
    pub messages_sent: u64,
    /// Messages received
    pub messages_received: u64,
    /// Errors encountered
    pub errors: u64,
    /// Connection uptime in seconds
    pub uptime_seconds: u64,
    /// Reconnect attempts
    pub reconnect_attempts: u32,
}

/// Message callback type
type MessageCallback = Arc<RwLock<dyn Fn(SpineMessage) -> Result<()> + Send + Sync>>;

impl SpineConnection {
    /// Create a new Spine connection
    pub fn new(config: SpineConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            connected: Arc::new(RwLock::new(false)),
            message_callbacks: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(SpineStats::default())),
        }
    }

    /// Connect to Mountain via Spine protocol
    #[instrument(skip(self))]
    pub async fn connect(&self) -> Result<()> {
        info!("Connecting to Spine at {}", self.config.protocol.mountain_endpoint);

        *self.state.write().await = ConnectionState::Connecting;

        // In real implementation, this would establish gRPC connection
        // For now, we simulate connection
        tokio::time::sleep(tokio::time::Duration::from_millis(
            self.config.protocol.connection_timeout_ms,
        ))
        .await;

        *self.connected.write().await = true;
        *self.state.write().await = ConnectionState::Connected;

        info!("Connected to Spine");

        Ok(())
    }

    /// Disconnect from Spine
    #[instrument(skip(self))]
    pub async fn disconnect(&self) -> Result<()> {
        info!("Disconnecting from Spine");

        *self.connected.write().await = false;
        *self.state.write().await = ConnectionState::Disconnected;

        info!("Disconnected from Spine");

        Ok(())
    }

    /// Send a message
    #[instrument(skip(self, message))]
    pub async fn send(&self, message: SpineMessage) -> Result<()> {
        if !*self.connected.read().await {
            return Err(anyhow::anyhow!("Not connected to Spine"));
        }

        debug!("Sending Spine message: {:?}", message.message_type);

        // In real implementation, this would send via gRPC
        // For now, we just update stats
        let mut stats = self.stats.write().await;
        stats.messages_sent += 1;

        Ok(())
    }

    /// Send a request and wait for response
    #[instrument(skip(self, payload))]
    pub async fn send_request<T: for<'de> Deserialize<'de>>(
        &self,
        payload: serde_json::Value,
    ) -> Result<T> {
        let message = SpineMessage::request(payload);
        self.send(message).await?;

        // In real implementation, this would wait for response
        // For now, we return an error
        Err(anyhow::anyhow!("Request not implemented"))
    }

    /// Register a message callback
    #[instrument(skip(self, callback))]
    pub async fn register_message_callback<F>(&self, callback: F)
    where
        F: Fn(SpineMessage) -> Result<()> + Send + Sync + 'static,
    {
        self.message_callbacks
            .write()
            .await
            .push(Arc::new(RwLock::new(callback)));
    }

    /// Get connection state
    pub async fn get_state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Get statistics
    pub async fn get_stats(&self) -> SpineStats {
        self.stats.read().await.clone()
    }

    /// Send heartbeat
    #[instrument(skip(self))]
    pub async fn send_heartbeat(&self) -> Result<()> {
        let message = SpineMessage::heartbeat();
        self.send(message).await?;
        Ok(())
    }

    /// Start heartbeat loop
    #[instrument(skip(self))]
    pub async fn start_heartbeat_loop(&self) -> Result<()> {
        let interval_sec = self.config.protocol.heartbeat_interval_sec;
        let connected = Arc::clone(&self.connected);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_sec)).await;
                if *connected.read().await {
                    // Send heartbeat
                    if let Err(e) = self.send_heartbeat().await {
                        warn!("Failed to send heartbeat: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Attempt to reconnect
    #[instrument(skip(self))]
    pub async fn reconnect(&self) -> Result<()> {
        let mut attempts = 0;
        let max_attempts = self.config.max_reconnect_attempts;
        let delay_ms = self.config.reconnect_delay_ms;

        *self.state.write().await = ConnectionState::Reconnecting;

        while attempts < max_attempts {
            attempts += 1;

            info!("Reconnection attempt {}/{}", attempts, max_attempts);

            match self.connect().await {
                Ok(_) => {
                    let mut stats = self.stats.write().await;
                    stats.reconnect_attempts = attempts;
                    return Ok(());
                }
                Err(e) => {
                    warn!("Reconnection attempt {} failed: {}", attempts, e);
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                }
            }
        }

        *self.state.write().await = ConnectionState::Error;

        Err(anyhow::anyhow!(
            "Failed to reconnect after {} attempts",
            max_attempts
        ))
    }
}

impl Default for SpineConnection {
    fn default() -> Self {
        Self::new(SpineConfig::new("grove-host".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spine_connection_creation() {
        let config = SpineConfig::new("test-host".to_string());
        let connection = SpineConnection::new(config);
        
        assert_eq!(
            connection.get_state().await,
            ConnectionState::Disconnected
        );
    }

    #[tokio::test]
    async fn test_spine_message_creation() {
        let message = SpineMessage::new(MessageType::Heartbeat);
        assert_eq!(message.message_type, MessageType::Heartbeat);
        assert!(!message.message_id.is_empty());
    }

    #[tokio::test]
    async fn test_spine_message_request() {
        let payload = serde_json::json!({ "test": "value" });
        let message = SpineMessage::request(payload);
        assert_eq!(message.message_type, MessageType::Request);
    }

    #[tokio::test]
    async fn test_spine_config_builder() {
        let config = SpineConfig::new("test".to_string())
            .with_auto_reconnect(false)
            .with_max_reconnect_attempts(5);

        assert_eq!(config.auto_reconnect, false);
        assert_eq!(config.max_reconnect_attempts, 5);
    }
}
