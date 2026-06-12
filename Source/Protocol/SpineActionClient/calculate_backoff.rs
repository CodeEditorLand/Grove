//! Calculate backoff delay based on attempt number
//!
//!  ☀️ 🟡 MOUNTAIN_GROVE_WASM

use crate::Protocol::SpineActionClient::ReconnectStrategy;

/// Calculate backoff delay based on attempt number
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
pub fn calculate_backoff(attempt:u32, strategy:&ReconnectStrategy) -> std::time::Duration {
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
