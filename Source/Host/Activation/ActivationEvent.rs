//! Extension activation event types

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Extension activation event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivationEvent {
	/// Activate when the extension host starts up
	Startup,

	/// Activate when a specific command is executed
	Command(String),

	/// Activate when a specific language is detected
	Language(String),

	/// Activate when a workspace of a specific type is opened
	WorkspaceContains(String),

	/// Activate when specific content type is viewed
	OnView(String),

	/// Activate when a URI scheme is used
	OnUri(String),

	/// Activate when specific file patterns match
	OnFiles(String),

	/// Custom activation event
	Custom(String),

	/// Activate on any event (always active)
	Star,
}

impl ActivationEvent {
	/// Parse an activation event from a string
	pub fn from_str(event_str:&str) -> Result<Self> {
		match event_str {
			"*" => Ok(Self::Star),

			e if e.starts_with("onCommand:") => Ok(Self::Command(e.trim_start_matches("onCommand:").to_string())),

			e if e.starts_with("onLanguage:") => Ok(Self::Language(e.trim_start_matches("onLanguage:").to_string())),

			e if e.starts_with("workspaceContains:") => {
				Ok(Self::WorkspaceContains(e.trim_start_matches("workspaceContains:").to_string()))
			},

			e if e.starts_with("onView:") => Ok(Self::OnView(e.trim_start_matches("onView:").to_string())),

			e if e.starts_with("onUri:") => Ok(Self::OnUri(e.trim_start_matches("onUri:").to_string())),

			e if e.starts_with("onFiles:") => Ok(Self::OnFiles(e.trim_start_matches("onFiles:").to_string())),

			_ => Ok(Self::Custom(event_str.to_string())),
		}
	}

	/// Convert to string representation
	pub fn to_string(&self) -> String {
		match self {
			Self::Startup => "onStartup".to_string(),

			Self::Star => "*".to_string(),

			Self::Command(cmd) => format!("onCommand:{}", cmd),

			Self::Language(lang) => format!("onLanguage:{}", lang),

			Self::WorkspaceContains(pattern) => format!("workspaceContains:{}", pattern),

			Self::OnView(view) => format!("onView:{}", view),

			Self::OnUri(uri) => format!("onUri:{}", uri),

			Self::OnFiles(pattern) => format!("onFiles:{}", pattern),

			Self::Custom(s) => s.clone(),
		}
	}
}

impl std::str::FromStr for ActivationEvent {
	type Err = anyhow::Error;

	fn from_str(s:&str) -> Result<Self, Self::Err> { Self::from_str(s) }
}
