use std::sync::Arc;

use serde_json;

use crate::Transport::Strategy::Transport;

/// Commands namespace
#[derive(Debug, Clone)]
pub struct CommandNamespace {
	/// Optional transport to Mountain for command forwarding.
	transport:Option<Arc<Transport>>,
}

impl CommandNamespace {
	/// Create a new CommandNamespace instance
	pub fn new() -> Self { Self { transport:None } }

	/// Create a CommandNamespace wired to a Mountain transport.
	pub fn new_with_transport(transport:Arc<Transport>) -> Self { Self { transport:Some(transport) } }

	/// Register a command
	pub fn register_command(&self, command_id:String, _callback:CommandCallback) -> Result<Command, String> {
		Ok(Command { id:command_id.clone() })
	}

	/// Execute a command by forwarding `commands:execute` to Mountain.
	/// When no transport is wired, returns an error so callers can fall back.
	pub async fn execute_command<T:serde::de::DeserializeOwned>(
		&self,

		command_id:String,

		args:Vec<serde_json::Value>,
	) -> Result<T, String> {
		let Some(transport) = &self.transport else {
			return Err(format!("No transport - cannot execute command: {}", command_id));
		};

		let payload = serde_json::to_vec(&serde_json::json!({
			"method": "commands:execute",
			"parameters": [command_id, args],
		}))
		.map_err(|e| format!("serialise: {}", e))?;

		let response = transport.send(&payload).await.map_err(|e| format!("transport: {}", e))?;

		serde_json::from_slice::<T>(&response).map_err(|e| format!("deserialise: {}", e))
	}
}

/// Command callback type
pub type CommandCallback = Box<dyn Fn(Vec<serde_json::Value>) -> Result<serde_json::Value, String> + Send + Sync>;

/// Command representation
#[derive(Debug, Clone)]
pub struct Command {
	/// The unique identifier of the command
	pub id:String,
}
