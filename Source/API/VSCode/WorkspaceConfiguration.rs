use std::sync::Arc;

use serde_json;

use crate::Transport::Strategy::Transport;

/// Workspace configuration
#[derive(Debug, Clone)]
pub struct WorkspaceConfiguration {
	/// The configuration section name
	section:Option<String>,

	/// Optional transport to Mountain for live configuration calls.
	transport:Option<Arc<Transport>>,
}

impl WorkspaceConfiguration {
	/// Create a new workspace configuration
	///
	/// # Arguments
	///
	/// * `section` - Optional section name to retrieve
	pub fn new(section:Option<String>) -> Self { Self { section, transport:None } }

	/// Create a workspace configuration wired to a Mountain transport.
	///
	/// # Arguments
	///
	/// * `section` - Optional section name to retrieve
	/// * `transport` - Active Mountain transport
	pub fn new_with_transport(section:Option<String>, transport:Arc<Transport>) -> Self {
		Self { section, transport:Some(transport) }
	}

	/// Get a configuration value (sync stub - returns Err; use
	/// `get_async` when a transport is available).
	pub fn get<T:serde::de::DeserializeOwned>(&self, _key:String) -> Result<T, String> {
		Err("Configuration not implemented".to_string())
	}

	/// Get a configuration value via Mountain `configuration:get`.
	pub async fn get_async<T:serde::de::DeserializeOwned>(&self, key:String) -> Result<T, String> {
		let Some(t) = &self.transport else {
			return Err("No transport wired".to_string());
		};

		let msg = serde_json::json!({
			"method": "configuration:get",
			"parameters": { "section": self.section, "key": key }
		});

		let bytes = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;

		let response = t.send(&bytes).await.map_err(|e| e.to_string())?;

		serde_json::from_slice::<T>(&response).map_err(|e| e.to_string())
	}

	/// Check if a key exists in the configuration
	pub fn has(&self, _key:String) -> bool { false }

	/// Update a configuration value via Mountain `configuration:update`
	/// (fire-and-forget).
	pub async fn update(&self, key:String, value:serde_json::Value) -> Result<(), String> {
		let Some(t) = &self.transport else {
			return Err("No transport wired".to_string());
		};

		let msg = serde_json::json!({
			"method": "configuration:update",
			"parameters": { "section": self.section, "key": key, "value": value }
		});

		let bytes = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;

		let t2 = Arc::clone(t);

		tokio::spawn(async move {
			let _ = t2.send_no_response(&bytes).await;
		});

		Ok(())
	}
}
