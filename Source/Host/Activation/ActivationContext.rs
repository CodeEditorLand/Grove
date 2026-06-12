//! Activation context passed to extensions

use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Activation context passed to extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationContext {
	/// Workspace root path
	pub workspace_path:Option<PathBuf>,

	/// Current file path
	pub current_file:Option<PathBuf>,

	/// Current language ID
	pub language_id:Option<String>,

	/// Active editor
	pub active_editor:bool,

	/// Environment variables
	pub environment:HashMap<String, String>,

	/// Additional context data
	pub additional_data:serde_json::Value,
}

impl Default for ActivationContext {
	fn default() -> Self {
		Self {
			workspace_path:None,

			current_file:None,

			language_id:None,

			active_editor:false,

			environment:HashMap::new(),

			additional_data:serde_json::Value::Null,
		}
	}
}
