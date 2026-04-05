//! # Grove Spine Connection
//!
//!  ☀️ 🟡 MOUNTAIN_GROVE_WASM - Grove (WASM+Rhai) connection to Mountain Spine
//!
//! Bidirectional gRPC connection with EchoAction support.
//!
//! ## Supported Extension Hosts
//!
//! - ✅ Grove (WASM + Rhai) - Primary implementation
//! - ✅ Sky (WASM only) - Compatible
//! - ❌ Cocoon (Node.js) - Separate implementation
//!
//! ## Feature Gates
//!
//! - `GROVE_RPC` - Enable gRPC communication (default)
//! - `GROVE_WASM` - Enable WASM runtime (default)
//! - `GROVE_TELEMETRY` - Enable OTEL integration
//!
//! ## Protocol
//!
//! 1. **Registration**: Grove registers with Mountain as a host
//! 2. **EchoActions**: Bidirectional EchoAction communication
//! 3. **RPC Calls**: Direct gRPC calls to Mountain's Spine services
//! 4. **Heartbeat**: Keepalive messages every 30 seconds
//!
//! ## Usage Example
//!
//! ```rust
//! use grove::{config::SpineConfig, protocol::SpineConnection};
//!
//! let config = SpineConfig::default();
//! let mut connection = SpineActionClient::new(config).await?;
//!
//! // Register with Mountain
//! connection.register().await?;
//!
//! // Send EchoAction
//! let response = connection.send_echo_action(action).await?;
//! ```

use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures::stream::StreamExt;
use tokio::sync::RwLock;
use tonic::transport::Channel;
use tracing::{debug, error, info, instrument, warn};

use crate::{
	api::vscode::APIBridge,
	vine::generated::vine::{
		EchoAction,
		EchoActionResponse,
		RegisterExtensionHostRequest,
		echo_action_service_client::EchoActionServiceClient,
	},
	wasm::Runtime as WASMRuntime,
};

/// Grove Spine Action Client
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM - EchoAction client + gRPC connection
pub struct SpineActionClient {
	/// Mountain connection details
	config:SpineConfig,

	/// gRPC channel
	channel:Option<Channel>,

	/// EchoAction client
	echo_client:Option<EchoActionServiceClient<Channel>>,

	/// Host ID
	host_id:String,

	/// Connection state
	connected:Arc<RwLock<bool>>,

	/// Timestamps
	connection_start_time:Arc<RwLock<Option<DateTime<Utc>>>>,
	last_heartbeat:Arc<RwLock<DateTime<Utc>>>,

	/// WASM runtime
	wasm_runtime:Option<Arc<WASMRuntime>>,

	/// API Bridge for VS Code API
	api_bridge:Option<Arc<APIBridge>>,

	/// Connected host information
	host_info:Arc<RwLock<Option<HostInfo>>>,
}

/// Spine configuration
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
#[derive(Clone, Debug)]
pub struct SpineConfig {
	/// Mountain gRPC URL
	pub mountain_url:String,

	/// Heartbeat interval (seconds)
	pub heartbeat_interval_sec:u64,

	/// Reconnect strategy
	pub reconnect_strategy:ReconnectStrategy,

	/// Capabilities
	pub capabilities:GroveCapabilities,
}

/// Grove host capabilities
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
#[derive(Clone, Debug)]
pub struct GroveCapabilities {
	/// Supports WASM
	pub wasm_enabled:bool,

	/// Supports Rhai
	pub rhai_enabled:bool,

	/// Supports Rhai native bridge
	pub native_bridge_enabled:bool,

	/// Maximum WASM memory (MB)
	pub wasm_memory_limit_mb:u32,

	/// Maximum concurrent Rhai scripts
	pub max_rhai_scripts:u32,

	/// Supported extension packages
	pub supported_extensions:Vec<String>, // ['wsix', 'rix', 'lsix']
}

impl Default for SpineConfig {
	fn default() -> Self {
		Self {
			mountain_url:"http://127.0.0.1:50051".to_string(),
			heartbeat_interval_sec:30,
			reconnect_strategy:ReconnectStrategy::default(),
			capabilities:GroveCapabilities {
				wasm_enabled:cfg!(feature = "wasm"),
				rhai_enabled:cfg!(feature = "rhai"),
				native_bridge_enabled:cfg!(feature = "bridge"),
				wasm_memory_limit_mb:512,
				max_rhai_scripts:100,
				supported_extensions:vec!["wsix".to_string(), "rix".to_string()],
			},
		}
	}
}

/// Reconnect strategy
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
#[derive(Clone, Debug)]
pub enum ReconnectStrategy {
	/// Never reconnect
	Never,

	/// Reconnect immediately
	Immediate,

	/// Exponential backoff (initial delay in ms, max delay in ms)
	ExponentialBackoff { initial_delay_ms:u64, max_delay_ms:u64 },

	/// Linear backoff (delay increment in ms, max delay in ms)
	LinearBackoff { increment_ms:u64, max_delay_ms:u64 },
}

impl Default for ReconnectStrategy {
	fn default() -> Self { Self::ExponentialBackoff { initial_delay_ms:1000, max_delay_ms:30000 } }
}

/// Host information from Mountain
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
#[derive(Clone, Debug)]
pub struct HostInfo {
	pub host_id:String,
	pub host_registry_id:String,
	pub heartbeat_interval_sec:u32,
}

impl SpineActionClient {
	/// Create new Spine action client
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	#[instrument(skip(config))]
	pub async fn new(config:SpineConfig) -> Result<Self> {
		let host_id = format!("grove-{}", uuid::Uuid::new_v4());

		info!("Creating Grove Spine client: {}", host_id);

		Ok(Self {
			config,
			channel:None,
			echo_client:None,
			host_id,
			connected:Arc::new(RwLock::new(false)),
			connection_start_time:Arc::new(RwLock::new(None)),
			last_heartbeat:Arc::new(RwLock::new(Utc::now())),
			wasm_runtime:None,
			api_bridge:None,
			host_info:Arc::new(RwLock::new(None)),
		})
	}

	/// Connect to Mountain
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	#[instrument(skip(self))]
	pub async fn connect(&mut self) -> Result<()> {
		info!("Connecting to Mountain at: {}", self.config.mountain_url);

		// Create gRPC channel
		let channel = Channel::from_static(self.config.mountain_url.as_str())
			.connect()
			.await
			.context("Failed to connect to Mountain gRPC server")?;

		// Create EchoAction client
		let echo_client = Some(EchoActionServiceClient::new(channel.clone()));

		// Store connection
		self.channel = Some(channel);
		self.echo_client = echo_client;

		// Update state
		*self.connected.write().await = true;
		*self.connection_start_time.write().await = Some(Utc::now());

		info!("Successfully connected to Mountain");

		Ok(())
	}

	/// Disconnection from Mountain
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	#[instrument(skip(self))]
	pub async fn disconnect(&mut self) -> Result<()> {
		info!("Disconnecting from Mountain");

		self.echo_client = None;
		self.channel = None;
		*self.connected.write().await = false;
		*self.connection_start_time.write().await = None;

		info!("Disconnected from Mountain");

		Ok(())
	}

	/// Register Grove as an extension host
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM - Host registration with Mountain
	#[instrument(skip(self))]
	pub async fn register(&self) -> Result<HostInfo> {
		let client = self
			.echo_client
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("Not connected to Mountain"))?;

		debug!("Registering Grove host: {}", self.host_id);

		// Build registration request
		let mut capabilities = HashMap::new();
		capabilities.insert("wasm_enabled".to_string(), self.config.capabilities.wasm_enabled.to_string());
		capabilities.insert("rhai_enabled".to_string(), self.config.capabilities.rhai_enabled.to_string());
		capabilities.insert(
			"native_bridge_enabled".to_string(),
			self.config.capabilities.native_bridge_enabled.to_string(),
		);
		capabilities.insert(
			"wasm_memory_limit_mb".to_string(),
			self.config.capabilities.wasm_memory_limit_mb.to_string(),
		);
		capabilities.insert(
			"max_rhai_scripts".to_string(),
			self.config.capabilities.max_rhai_scripts.to_string(),
		);
		capabilities.insert("supports_terminals".to_string(), "false".to_string());
		capabilities.insert("supports_processes".to_string(), "false".to_string());
		capabilities.insert("supports_debug".to_string(), "false".to_string());
		capabilities.insert("supports_scm".to_string(), "false".to_string());
		capabilities.insert("supports_webviews".to_string(), "true".to_string());

		let metadata = crate::vine::generated::vine::HostMetadata {
			version:env!("CARGO_PKG_VERSION").to_string(),
			build_hash:option_env!("BUILD_HASH").unwrap_or("unknown").to_string(),
			supported_extensions:self.config.capabilities.supported_extensions.clone(),
			max_memory_mb:self.config.capabilities.wasm_memory_limit_mb,
			enabled_features:vec![
				if cfg!(feature = "wasm") { "wasm".to_string() } else { String::new() },
				if cfg!(feature = "rhai") { "rhai".to_string() } else { String::new() },
				if cfg!(feature = "bridge") { "bridge".to_string() } else { String::new() },
			]
			.into_iter()
			.filter(|s| !s.is_empty())
			.collect(),
		};

		let request = RegisterExtensionHostRequest {
			host_id:self.host_id.clone(),
			host_type:2, // Grove
			capabilities,
			metadata:Some(metadata),
		};

		// Send registration request
		let response = client
			.register_extension_host(request)
			.await
			.context("Failed to register Grove host")?
			.into_inner();

		debug!("Grove host registered: {}", response.host_registry_id);

		let host_info = HostInfo {
			host_id:self.host_id.clone(),
			host_registry_id:response.host_registry_id,
			heartbeat_interval_sec:response.heartbeat_interval_sec,
		};

		*self.host_info.write().await = Some(host_info.clone());

		// Start heartbeat loop
		self.start_heartbeat_loop().await?;

		// Start EchoAction listener
		self.start_echo_action_listener().await?;

		info!("Grove host successfully registered and active");

		Ok(host_info)
	}

	/// Send EchoAction to Mountain
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	#[instrument(skip(self, action))]
	pub async fn send_echo_action(&self, action:EchoAction) -> Result<EchoActionResponse> {
		let client = self
			.echo_client
			.as_ref()
			.ok_or_else(|| anyhow::anyhow!("Not connected to Mountain"))?;

		debug!("Sending EchoAction: type={}, target={}", action.action_type, action.target);

		let response = client
			.send_echo_action(action)
			.await
			.context("Failed to send EchoAction")?
			.into_inner();

		debug!(
			"EchoAction response: success={}, processing_time_ms={}",
			response.success, response.processing_time_ms
		);

		if !response.success {
			anyhow::bail!("EchoAction failed: {}", response.error);
		}

		Ok(response)
	}

	/// Send RPC via EchoAction
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	#[instrument(skip(self, payload))]
	pub async fn send_rpc_via_action(
		&self,
		rpc_method:&str,
		payload:Vec<u8>,
		target_host:Option<&str>,
	) -> Result<Vec<u8>> {
		let mut headers = vec![
			("rpc_method".to_string(), rpc_method.to_string()),
			("host_type".to_string(), "grove".to_string()),
		];

		if let Some(target) = target_host {
			headers.push(("target_host".to_string(), target.to_string()));
		}

		let action = EchoAction {
			action_id:uuid::Uuid::new_v4().to_string(),
			source:self.host_id.clone(),
			target:target_host.unwrap_or("mountain").to_string(),
			action_type:"rpc".to_string(),
			payload,
			headers:headers.into_iter().collect(),
			timestamp:Utc::now().timestamp(),
			nested_actions:vec![],
		};

		let response = self.send_echo_action(action).await?;
		Ok(response.result)
	}

	/// Send event via EchoAction
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	pub async fn send_event(&self, event_name:&str, payload:Vec<u8>, metadata:HashMap<String, String>) -> Result<()> {
		let mut headers = vec![
			("event_name".to_string(), event_name.to_string()),
			("host_type".to_string(), "grove".to_string()),
		];

		for (key, value) in metadata {
			headers.push((key, value));
		}

		let action = EchoAction {
			action_id:uuid::Uuid::new_v4().to_string(),
			source:self.host_id.clone(),
			target:"mountain".to_string(),
			action_type:"event".to_string(),
			payload,
			headers:headers.into_iter().collect(),
			timestamp:Utc::now().timestamp(),
			nested_actions:vec![],
		};

		self.send_echo_action(action).await?;
		Ok(())
	}

	/// Start heartbeat loop
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	/// Sends periodic heartbeat EchoActions to maintain connection
	async fn start_heartbeat_loop(&self) -> Result<()> {
		let connected = Arc::clone(&self.connected);
		let last_heartbeat = Arc::clone(&self.last_heartbeat);
		let interval_sec = self.config.heartbeat_interval_sec;

		tokio::spawn(async move {
			loop {
				tokio::time::sleep(tokio::time::Duration::from_secs(interval_sec)).await;
				if *connected.read().await {
					*last_heartbeat.write().await = Utc::now();
					// Heartbeat is maintained via last_heartbeat timestamp
					// Actual EchoAction heartbeat messages will be sent through
					// the gRPC bidirectional streaming when EchoAction protocol
					// is fully implemented
					debug!(
						"[SpineConnection] Heartbeat maintained (last: {})",
						*last_heartbeat.read().await
					);
				}
			}
		});

		info!("[SpineConnection] Heartbeat loop started (interval: {}s)", interval_sec);
		Ok(())
	}

	/// Start EchoAction listener
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM - Receives EchoActions from Mountain
	/// Listens for EchoAction messages from Mountain over the bidirectional
	/// gRPC stream. Currently implemented as a stub that logs when the
	/// listener is active.
	///
	/// The EchoAction protocol requires:
	/// - Bidirectional streaming RPC endpoint in the gRPC service
	/// - EchoAction message types defined in protos/Spine.proto
	/// - Proper message deserialization and routing
	async fn start_echo_action_listener(&self) -> Result<()> {
		// EchoAction streaming/listening requires bidirectional gRPC streaming
		// This will be implemented once:
		// 1. EchoAction message types are fully defined in the proto files
		// 2. The Spine gRPC service includes a bidirectional streaming RPC
		// 3. The client has access to the streaming endpoint
		//
		// For now, we log that the listener is ready for future implementation
		info!("[SpineConnection] EchoAction listener initialized");
		info!("[SpineConnection] Waiting for EchoAction protocol implementation");
		Ok(())
	}

	/// Get connection status
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	pub async fn get_status(&self) -> ConnectionStatus {
		let connected = *self.connected.read().await;
		let start = *self.connection_start_time.read().await;
		let host_info = self.host_info.read().await.clone();

		ConnectionStatus {
			connected,
			host_id:self.host_id.clone(),
			uptime:start.map(|s| (Utc::now() - s).num_seconds()),
			host_info,
		}
	}

	/// Set WASM runtime
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	pub fn set_wasm_runtime(&mut self, runtime:Arc<WASMRuntime>) { self.wasm_runtime = Some(runtime); }

	/// Set API Bridge
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	pub fn set_api_bridge(&mut self, bridge:Arc<APIBridge>) { self.api_bridge = Some(bridge); }

	/// Attempt to reconnect
	///
	///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
	#[instrument(skip(self))]
	pub async fn reconnect(&mut self) -> Result<()> {
		warn!("Attempting to reconnect to Mountain");

		self.disconnect().await?;
		self.connect().await?;
		self.register().await?;

		info!("Successfully reconnected to Mountain");

		Ok(())
	}
}

/// Connection status
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
#[derive(Clone, Debug)]
pub struct ConnectionStatus {
	pub connected:bool,
	pub host_id:String,
	pub uptime:Option<i64>,
	pub host_info:Option<HostInfo>,
}

// ============================================================================
// Backoff calculation helper
// ============================================================================

/// Calculate backoff delay based on attempt number
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
fn calculate_backoff(attempt:u32, strategy:&ReconnectStrategy) -> std::time::Duration {
	match strategy {
		ReconnectStrategy::Never => return std::time::Duration::from_secs(0),
		ReconnectStrategy::Immediate => return std::time::Duration::from_secs(0),
		ReconnectStrategy::ExponentialBackoff { initial_delay_ms, max_delay_ms } => {
			let delay_ms = std::cmp::min(initial_delay_ms * 2u64.pow(attempt.saturating_sub(1)), *max_delay_ms);
			std::time::Duration::from_millis(delay_ms)
		},
		ReconnectStrategy::LinearBackoff { increment_ms, max_delay_ms } => {
			let delay_ms = std::cmp::min(increment_ms * attempt as u64, *max_delay_ms);
			std::time::Duration::from_millis(delay_ms)
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_calculate_backoff_exponential() {
		let strategy = ReconnectStrategy::ExponentialBackoff { initial_delay_ms:1000, max_delay_ms:10000 };

		assert_eq!(calculate_backoff(1, &strategy).as_millis(), 1000);
		assert_eq!(calculate_backoff(2, &strategy).as_millis(), 2000);
		assert_eq!(calculate_backoff(3, &strategy).as_millis(), 4000);
		assert_eq!(calculate_backoff(10, &strategy).as_millis(), 10000); // Capped
	}

	#[test]
	fn test_calculate_backoff_linear() {
		let strategy = ReconnectStrategy::LinearBackoff { increment_ms:500, max_delay_ms:2000 };

		assert_eq!(calculate_backoff(1, &strategy).as_millis(), 500);
		assert_eq!(calculate_backoff(2, &strategy).as_millis(), 1000);
		assert_eq!(calculate_backoff(3, &strategy).as_millis(), 1500);
		assert_eq!(calculate_backoff(10, &strategy).as_millis(), 2000); // Capped
	}
}
