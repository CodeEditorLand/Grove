//! Spine configuration
//!
//!  ☀️ 🟡 MOUNTAIN_GROVE_WASM

use crate::Protocol::SpineActionClient::{GroveCapabilities, ReconnectStrategy};

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
