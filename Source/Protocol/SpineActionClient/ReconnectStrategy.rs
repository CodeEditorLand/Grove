//! Reconnect strategy
//!
//!  ☀️ 🟡 MOUNTAIN_GROVE_WASM

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
