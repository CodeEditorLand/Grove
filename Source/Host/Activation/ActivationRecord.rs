//! Activation record for tracking

use serde::{Deserialize, Serialize};

/// Activation record for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationRecord {
	/// Extension ID
	pub extension_id:String,

	/// Activation events
	pub events:Vec<String>,

	/// Activation time (Unix timestamp)
	pub timestamp:u64,

	/// Duration in milliseconds
	pub duration_ms:u64,

	/// Success flag
	pub success:bool,

	/// Error message (if failed)
	pub error:Option<String>,
}
