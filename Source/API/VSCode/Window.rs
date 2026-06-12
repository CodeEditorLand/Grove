use std::sync::Arc;

use serde_json;

use crate::Transport::Strategy::Transport;
use super::OutputChannel::OutputChannel;

/// Window namespace
#[derive(Debug, Clone)]
pub struct Window {
	/// Transport for forwarding notifications to Mountain
	transport:Option<Arc<Transport>>,
}

impl Window {
	/// Create a new Window instance
	pub fn new() -> Self { Self { transport:None } }

	/// Create a Window instance with a transport for notification forwarding
	pub fn new_with_transport(transport:Arc<Transport>) -> Self { Self { transport:Some(transport) } }

	/// Show an information message
	pub async fn show_information_message(&self, message:String) -> Result<String, String> {
		if let Some(t) = &self.transport {
			let msg =
				serde_json::json!({"method":"notification:show","parameters":{"message":message,"severity":"info"}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}

		Ok("OK".to_string())
	}

	/// Show a warning message
	pub async fn show_warning_message(&self, message:String) -> Result<String, String> {
		if let Some(t) = &self.transport {
			let msg =
				serde_json::json!({"method":"notification:show","parameters":{"message":message,"severity":"warning"}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}

		Ok("OK".to_string())
	}

	/// Show an error message
	pub async fn show_error_message(&self, message:String) -> Result<String, String> {
		if let Some(t) = &self.transport {
			let msg =
				serde_json::json!({"method":"notification:show","parameters":{"message":message,"severity":"error"}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}

		Ok("OK".to_string())
	}

	/// Create and show a new output channel
	pub fn create_output_channel(&self, name:String) -> OutputChannel {
		match &self.transport {
			Some(t) => OutputChannel::new_with_transport(name, Arc::clone(t)),

			None => OutputChannel::new(name),
		}
	}
}
