use std::sync::Arc;

use serde_json;

use crate::{Transport::Strategy::Transport, dev_log};

/// Output channel for logging
#[derive(Debug, Clone)]
pub struct OutputChannel {
	/// The name of the output channel
	name:String,

	/// Transport for forwarding show/hide/dispose notifications to Mountain
	transport:Option<Arc<Transport>>,
}

impl OutputChannel {
	/// Create a new output channel
	///
	/// # Arguments
	///
	/// * `name` - The name of the output channel
	pub fn new(name:String) -> Self { Self { name, transport:None } }

	/// Create a new output channel with a transport for notification forwarding
	pub fn new_with_transport(name:String, transport:Arc<Transport>) -> Self {
		Self { name, transport:Some(transport) }
	}

	/// Append a line to the channel
	pub fn append_line(&self, line:&str) {
		dev_log!("output", "[{}] {}", self.name, line);
	}

	/// Append to the channel
	pub fn append(&self, value:&str) {
		dev_log!("output", "[{}] {}", self.name, value);
	}

	/// Show the output channel
	pub fn show(&self) {
		if let Some(t) = &self.transport {
			let msg = serde_json::json!({"method":"output:show","parameters":{"channel":self.name}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}
	}

	/// Hide the output channel
	pub fn hide(&self) {
		if let Some(t) = &self.transport {
			let msg = serde_json::json!({"method":"output:hide","parameters":{"channel":self.name}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}
	}

	/// Dispose the output channel
	pub fn dispose(&self) {
		if let Some(t) = &self.transport {
			let msg = serde_json::json!({"method":"output:dispose","parameters":{"channel":self.name}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}
	}
}
