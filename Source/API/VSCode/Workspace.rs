use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::Transport::Strategy::Transport;
use crate::dev_log;

use super::WorkspaceConfiguration::WorkspaceConfiguration;

/// Workspace namespace
#[derive(Debug, Clone)]
pub struct Workspace {
	/// Optional transport to Mountain for configuration calls.
	transport:Option<Arc<Transport>>,
}

impl Workspace {
	/// Create a new Workspace instance
	pub fn new() -> Self { Self { transport:None } }

	/// Create a Workspace wired to a Mountain transport.
	pub fn new_with_transport(transport:Arc<Transport>) -> Self { Self { transport:Some(transport) } }

	/// Get workspace folders (sync).
	///
	/// Returns an empty vec when called from a synchronous context. Use
	/// `workspace_folders_async` from an async context to retrieve live data
	/// from Mountain.
	pub fn workspace_folders(&self) -> Vec<WorkspaceFolder> {
		dev_log!(
			"workspace",
			"[Workspace::workspace_folders] transport wired but called synchronously"
		);

		Vec::new()
	}

	/// Get workspace folders by querying Mountain via the transport.
	///
	/// Falls back to an empty vec on any transport or parse error.
	pub async fn workspace_folders_async(&self) -> Vec<WorkspaceFolder> {
		let Some(t) = &self.transport else {
			return Vec::new();
		};

		let msg = serde_json::json!({"method":"workspaces:getFolders","parameters":{}});

		let Ok(bytes) = serde_json::to_vec(&msg) else {
			return Vec::new();
		};

		match t.send(&bytes).await {
			Ok(response) => serde_json::from_slice::<Vec<WorkspaceFolder>>(&response).unwrap_or_default(),

			Err(e) => {
				dev_log!("workspace", "[workspace_folders_async] error: {}", e);

				Vec::new()
			},
		}
	}

	/// Get workspace configuration
	pub fn get_configuration(&self, section:Option<String>) -> WorkspaceConfiguration {
		match &self.transport {
			Some(t) => WorkspaceConfiguration::new_with_transport(section, Arc::clone(t)),

			None => WorkspaceConfiguration::new(section),
		}
	}
}

/// Workspace folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
	/// The uri of the workspace folder
	pub uri:String,

	/// The name of the workspace folder
	pub name:String,

	/// The ordinal number of the workspace folder
	pub index:u32,
}
