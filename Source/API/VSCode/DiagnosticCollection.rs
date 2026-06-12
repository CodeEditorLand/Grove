use std::sync::Arc;

use serde_json;

use crate::{API::Types::Diagnostic, Transport::Strategy::Transport};

/// Diagnostic collection
#[derive(Debug, Clone)]
pub struct DiagnosticCollection {
	/// The name of the diagnostic collection
	name:Option<String>,

	/// Optional transport to Mountain for forwarding diagnostic notifications.
	transport:Option<Arc<Transport>>,
}

impl DiagnosticCollection {
	/// Create a new diagnostic collection
	///
	/// # Arguments
	///
	/// * `name` - Optional name for the collection
	pub fn new(name:Option<String>) -> Self { Self { name, transport:None } }

	/// Create a new diagnostic collection wired to a Mountain transport.
	/// set/delete/clear/dispose calls are forwarded via `send_no_response`.
	pub fn new_with_transport(name:Option<String>, transport:Arc<Transport>) -> Self {
		Self { name, transport:Some(transport) }
	}

	/// Forward a notification to Mountain if a transport is wired.
	fn fire(&self, method:&str, params:serde_json::Value) {
		if let Some(t) = &self.transport {
			let msg = serde_json::json!({"method": method, "parameters": params});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}
	}

	/// Set diagnostics for a resource
	pub fn set(&self, uri:String, diagnostics:Vec<Diagnostic>) {
		self.fire("diagnostics:set", serde_json::json!({"uri": uri, "diagnostics": diagnostics}));
	}

	/// Delete diagnostics for a resource
	pub fn delete(&self, uri:String) { self.fire("diagnostics:delete", serde_json::json!({"uri": uri})); }

	/// Clear all diagnostics in this collection
	pub fn clear(&self) { self.fire("diagnostics:clear", serde_json::json!({"name": self.name})); }

	/// Dispose the collection and release Mountain-side resources
	pub fn dispose(&self) { self.fire("diagnostics:dispose", serde_json::json!({"name": self.name})); }
}
