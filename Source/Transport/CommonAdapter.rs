//! # Common Transport Adapter
//!
//! Adapter that bridges Grove's transport implementations to the Common
//! `TransportStrategy` trait defined in the Common crate.
//!
//! This adapter allows Grove's existing `GrpcTransport`, `IPCTransportImpl`,
//! and `WASMTransportImpl` to be used through the unified Common transport
//! interface, enabling transport-agnostic code in the application.

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use CommonLibrary::{
	Environment::Environment,
	Transport::{
		TransportCapabilities,
		TransportConfig,
		TransportError,
		TransportMetrics,
		TransportStrategy,
		TransportType as CommonTransportType,
		UnifiedRequest,
		UnifiedResponse,
	},
};

use crate::Transport::{
	GrpcTransport as GroveGrpcTransport,
	IPCTransportImpl as GroveIpcTransport,
	Transport as GroveTransport,
	TransportConfig as GroveTransportConfig,
	TransportStats as GroveTransportStats,
	TransportStrategy as GroveTransportStrategy,
	TransportType as GroveTransportType,
	WASMTransportImpl as GroveWasmTransport,
};

/// Adapter that wraps a Grove `Transport` enum and presents it as a Common
/// `TransportStrategy`.
///
/// This is the bridge between the Common abstraction layer and Grove's concrete
/// transport implementations. It translates between the Common DTOs
/// (`UnifiedRequest`, `UnifiedResponse`, `TransportError`) and Grove's types
/// (`TransportMessage`, `TransportStats`, etc.).
///
/// # Usage
///
/// ```rust,ignore
/// use common_common::transport::TransportStrategy;
///
/// let grove_transport = Transport::gRPC(GrpcTransport::new("127.0.0.1:50050")?);
/// let adapter = TransportAdapter::new(grove_transport).await?;
///
/// // Now use through Common trait
/// let request = UnifiedRequest::new("fileSystem.readFile")
///     .with_payload(serde_json::to_vec(&params)?);
/// let response = adapter.send_request(request).await?;
/// ```
#[derive(Clone, Debug)]
pub struct TransportAdapter {
	/// The underlying Grove transport (wrapped in Arc for thread sharing)
	transport:Arc<GroveTransport>,

	/// Grove-side transport configuration
	config:GroveTransportConfig,

	/// Common-library TransportConfig view (built from grove config at
	/// construction)
	common_config:TransportConfig,

	/// Correlation ID generator (use default UUID generator)
	correlation_generator:fn() -> String,
}

impl TransportAdapter {
	/// Creates a new `TransportAdapter` from a Grove `Transport`.
	///
	/// # Parameters
	///
	/// * `transport` - The Grove transport to wrap
	///
	/// # Returns
	///
	/// A new `TransportAdapter` ready for use as a Common `TransportStrategy`.
	pub fn new(transport:GroveTransport) -> Self {
		let Config = GroveTransportConfig::default();

		let CommonConfig = Self::BuildCommonConfig(&Config);

		Self {
			transport:Arc::new(transport),

			config:Config,

			common_config:CommonConfig,

			correlation_generator:|| uuid::Uuid::new_v4().to_string(),
		}
	}

	/// Creates a new `TransportAdapter` with custom configuration.
	pub fn with_config(transport:GroveTransport, config:GroveTransportConfig) -> Self {
		let CommonConfig = Self::BuildCommonConfig(&config);

		Self {
			transport:Arc::new(transport),

			config,

			common_config:CommonConfig,

			correlation_generator:|| uuid::Uuid::new_v4().to_string(),
		}
	}

	/// Builds a `CommonLibrary::Transport::TransportConfig` from a
	/// `GroveTransportConfig`, mapping the overlapping fields.
	fn BuildCommonConfig(Grove:&GroveTransportConfig) -> TransportConfig {
		TransportConfig {
			DefaultTimeout:Grove.RequestTimeout,

			MaximumRetries:Grove.MaximumRetries,

			RetryBaseDelay:Grove.RetryDelay,

			RetryMaximumDelay:Grove.RetryDelay * 10,

			RetryJitterEnabled:true,

			CircuitBreakerFailureThreshold:5,

			CircuitBreakerResetTimeout:Duration::from_secs(30),

			HealthChecksEnabled:true,

			HealthCheckInterval:Grove.KeepaliveInterval,

			MetricsEnabled:true,

			TransportConfigurations:HashMap::new(),
			..TransportConfig::default()
		}
	}

	/// Gets the underlying Grove transport.
	pub fn grove_transport(&self) -> &GroveTransport { &self.transport }

	/// Gets the Grove-side transport configuration.
	pub fn config(&self) -> &GroveTransportConfig { &self.config }

	/// Converts a Grove `TransportType` to a Common `TransportType`.
	fn translate_transport_type(grove_type:GroveTransportType) -> CommonTransportType {
		match grove_type {
			GroveTransportType::gRPC => CommonTransportType::Grpc,

			GroveTransportType::IPC => CommonTransportType::Ipc,

			GroveTransportType::WASM => CommonTransportType::Wasm,

			GroveTransportType::Unknown => CommonTransportType::Unknown,
		}
	}

	/// Converts a Grove `TransportStats` to a Common `TransportMetrics`.
	fn translate_metrics(stats:GroveTransportStats) -> TransportMetrics {
		TransportMetrics {
			requests_total:stats.messages_sent + stats.messages_received,

			requests_successful:stats.messages_received, // Assume received = successful for now

			requests_failed:stats.errors,

			notifications_sent:0, // TODO: track separately

			connections_established:1, // Assume 1 connection

			connection_failures:if stats.errors > 0 { 1 } else { 0 },

			bytes_sent:stats.bytes_sent,

			bytes_received:stats.bytes_received,

			circuit_breaker_state:1, // Assume closed (1)

			latency_ms_histogram:stats
				.avg_latency_us
				.map(|us| (1, us as f64 / 1000.0, (us as f64 / 1000.0).powi(2))),

			active_connections:1,

			pending_requests:0,
		}
	}
}

#[async_trait]
impl TransportStrategy for TransportAdapter {
	async fn connect(&mut self) -> Result<(), TransportError> {
		self.transport.connect().await.map_err(|e| {
			TransportError::connection(format!("Failed to connect transport: {}", e)).with_transport_type("grove")
		})
	}

	async fn disconnect(&mut self) -> Result<(), TransportError> {
		self.transport.close().await.map_err(|e| {
			TransportError::connection(format!("Failed to disconnect transport: {}", e)).with_transport_type("grove")
		})
	}

	async fn send_request(&mut self, request:UnifiedRequest) -> Result<UnifiedResponse, TransportError> {
		// Validate request
		request.validate().map_err(|e| {
			TransportError::invalid_request(format!("Invalid request: {}", e)).with_transport_type("grove")
		})?;

		// Determine timeout
		let timeout = request
			.timeout_ms()
			.map(std::time::Duration::from_millis)
			.unwrap_or(self.config.request_timeout);

		// Serialize request as a JSON envelope: {"method": method, "payload":
		// base64(payload)} so the receiver can dispatch to the right handler by
		// method name.
		let payload_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &request.payload);

		let envelope = serde_json::json!({
			"method": request.method,
			"correlationId": request.correlation_id,
			"payload": payload_b64,
		});

		let request_data = serde_json::to_vec(&envelope).map_err(|e| {
			TransportError::serialization(format!("envelope serialisation: {}", e)).with_transport_type("grove")
		})?;

		let result = self
			.transport
			.send(&request_data)
			.await
			.map_err(|e| TransportError::from(e).with_transport_type("grove"));

		// Build response
		match result {
			Ok(response_bytes) => {
				let correlation_id = request.correlation_id.unwrap_or_else(|| (self.correlation_generator)());

				Ok(UnifiedResponse::success(correlation_id, response_bytes))
			},

			Err(e) => {
				let correlation_id = request.correlation_id.unwrap_or_else(|| (self.correlation_generator)());

				Ok(UnifiedResponse::from_transport_error(correlation_id, &e))
			},
		}
	}

	async fn send_notification(&mut self, notification:UnifiedRequest) -> Result<(), TransportError> {
		notification.validate().map_err(|e| {
			TransportError::invalid_request(format!("Invalid notification: {}", e)).with_transport_type("grove")
		})?;

		// Serialize notification
		let mut data = Vec::new();

		data.extend_from_slice(&notification.payload);

		self.transport
			.send_no_response(&data)
			.await
			.map_err(|e| TransportError::from(e).with_transport_type("grove"))
	}

	fn stream_events(
		&self,
	) -> std::result::Result<futures::stream::BoxStream<'static, UnifiedResponse>, TransportError> {
		// Grove transports currently don't support streaming in the same way.
		// This would need to be implemented if any transport supports server push.
		Err(TransportError::not_supported("Streaming not implemented in Grove transports"))
	}

	fn is_connected(&self) -> bool {
		// Unfortunately Grove's `is_connected` requires `&self`, not `&self` with
		// interior mutability? Let's check the trait again - it takes `&self` which
		// is fine. But `Transport` enum's `is_connected` method takes `&self`, which
		// is also fine. However, we have `Arc<Transport>`, so we can call it.
		// Wait, `Transport` enum's `is_connected` method is defined on `&self` and
		// doesn't require mut. But we have `transport: Arc<GroveTransport>`, we can
		// call `self.transport.is_connected()`. But in the method signature above we
		// have `self.transport` as `Arc<GroveTransport>`, not `&GroveTransport`.
		// We need to deref.
		// Let's check: `self.transport.is_connected()` should work because
		// `Arc<GroveTransport>` derefs to `GroveTransport`. But `is_connected` takes
		// `&self`. `Arc` deref to `T`, so `&self.transport` is `&&GroveTransport`,
		// that's fine. Actually `self.transport.is_connected()` means we're calling
		// `is_connected` on `Arc<GroveTransport>`, which uses deref coercion to call
		// `is_connected` on `&GroveTransport`? Let's think: `Arc<T>` implements
		// `Deref<Target = T>`, so `&Arc<T>` can be used as `&T`. But here we have
		// `self.transport` which is `Arc<GroveTransport>`, not `&Arc<...>`.
		// So we need to either do `(&*self.transport).is_connected()` or simply
		// `self.transport.is_connected()` doesn't work because `is_connected` expects
		// `&self` and we have an owned `Arc`. Actually `Arc<T>` does not automatically
		// deref to `&T` when you have an owned `Arc`. You need to pass a reference:
		// `(&*self.transport).is_connected()`. So I'll fix:
		// `(&*self.transport).is_connected()`
		(&*self.transport).is_connected()
	}

	fn latency_ms(&self) -> u64 {
		// Grove doesn't expose latency directly; we could estimate from stats
		// For now return 0 or compute from avg_latency_us if available
		// We'd need to fetch stats which is async. This method is sync, so we can't
		// easily get it. We could store cached metrics, but for now return 0.
		// TODO: Implement proper metrics caching if needed
		0
	}

	fn transport_type(&self) -> CommonTransportType {
		let grove_type = self.transport.transport_type();

		Self::translate_transport_type(grove_type)
	}

	fn config(&self) -> &TransportConfig {
		// common_config is built from GroveTransportConfig at construction time
		// via TransportAdapter::BuildCommonConfig.
		&self.common_config
	}

	fn supports_streaming(&self) -> bool {
		// Grove's transports don't support streaming currently
		false
	}

	fn capabilities(&self) -> TransportCapabilities {
		TransportCapabilities {
			max_message_size:1024 * 1024, // 1MB

			supports_request_response:true,

			supports_server_streaming:false,

			supports_client_streaming:false,

			supports_bidirectional_streaming:false,

			supports_notifications:true,

			max_concurrent:100,

			requires_network:self.transport.transport_type() != GroveTransportType::IPC,

			supports_encryption:false,

			supports_compression:false,
		}
	}

	fn metrics(&self) -> TransportMetrics {
		// This is async in Grove but sync here. We can't block. We could spawn a
		// blocking task but that would require runtime. For now, return empty or
		// cached metrics. We could store metrics in an Arc<Mutex> that we update
		// periodically. That's more complex. For a simple adapter, we might change
		// the trait method to async, but it's defined as sync. Let's return a
		// default.
		TransportMetrics::new()
	}
}

// Implement conversion from Grove's TransportError to Common's TransportError
impl From<crate::Transport::GrpcTransportError> for TransportError {
	fn from(err:crate::Transport::GrpcTransportError) -> Self {
		match err {
			crate::Transport::GrpcTransportError::ConnectionFailed(msg) => TransportError::connection(msg),

			crate::Transport::GrpcTransportError::NotConnected => TransportError::connection("Not connected"),

			crate::Transport::GrpcTransportError::SendFailed(msg) => {
				TransportError::new(
					super::TransportErrorCode::MessageTooLarge, // Actually send failed
					msg,
				)
			},

			crate::Transport::GrpcTransportError::Timeout => TransportError::timeout("Operation timed out"),

			_ => TransportError::internal(format!("gRPC transport error: {}", err)),
		}
	}
}

// Similarly for other error types if they exist
// TODO: Implement From<IPCTransportError> and From<WasmTransportError> when
// those are defined

#[cfg(test)]
mod tests {

	use super::*;

	#[tokio::test]
	async fn test_adapter_creation() {
		let grove_transport = GroveTransport::gRPC(GrpcTransport::new("127.0.0.1:50050").unwrap());

		let adapter = TransportAdapter::new(grove_transport);

		assert_eq!(adapter.transport_type(), CommonTransportType::Grpc);
	}
}
