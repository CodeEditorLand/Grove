//! Activation Module
//!
//! Handles extension activation events and orchestration.
//! Manages the activation lifecycle for extensions.

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::{
    Host::{ActivationResult, HostConfig},
    Host::ExtensionManager::{ExtensionManagerImpl, ExtensionState},
};

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

/// Activation engine for managing extension activation
pub struct ActivationEngine {
    /// Extension manager
    extension_manager:Arc<ExtensionManagerImpl>,
    /// Host configuration
    #[allow(dead_code)]
    config:HostConfig,
    /// Event handlers mapping
    event_handlers:Arc<RwLock<HashMap<String, ActivationHandler>>>,
    /// Activation history
    activation_history:Arc<RwLock<Vec<ActivationRecord>>>,
}

/// Activation handler for an extension
#[derive(Debug, Clone)]
struct ActivationHandler {
    /// Extension ID
    #[allow(dead_code)]
    extension_id:String,
    /// Activation events
    events:Vec<ActivationEvent>,
    /// Activation function path
    #[allow(dead_code)]
    activation_function:String,
    /// Whether extension is currently active
    is_active:bool,
    /// Last activation time
    #[allow(dead_code)]
    last_activation:Option<u64>,
}

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

impl ActivationEngine {
	/// Create a new activation engine
	pub fn new(extension_manager:Arc<ExtensionManagerImpl>, config:HostConfig) -> Self {
		Self {
			extension_manager,
			config,
			event_handlers:Arc::new(RwLock::new(HashMap::new())),
			activation_history:Arc::new(RwLock::new(Vec::new())),
		}
	}

	/// Activate an extension
	#[instrument(skip(self, extension_id))]
	pub async fn activate(&self, extension_id:&str) -> Result<ActivationResult> {
		info!("Activating extension: {}", extension_id);

		let start = std::time::Instant::now();

		// Get extension info
		let extension_info = self
			.extension_manager
			.get_extension(extension_id)
			.await
			.ok_or_else(|| anyhow::anyhow!("Extension not found: {}", extension_id))?;

		// Check if already active
		let handlers = self.event_handlers.read().await;
		if let Some(handler) = handlers.get(extension_id) {
			if handler.is_active {
				warn!("Extension already active: {}", extension_id);
				return Ok(ActivationResult {
					extension_id:extension_id.to_string(),
					success:true,
					time_ms:0,
					error:None,
					contributes:Vec::new(),
				});
			}
		}
		drop(handlers);

		// Parse activation events
		let activation_events:Result<Vec<ActivationEvent>> = extension_info
			.activation_events
			.iter()
			.map(|e| ActivationEvent::from_str(e))
			.collect();
		let activation_events = activation_events.with_context(|| "Failed to parse activation events")?;

		// Create activation context
		let context = ActivationContext::default();

		// Perform activation (in real implementation, this would call the extension's
		// activate function)
		let activation_result = self
			.perform_activation(extension_id, &context)
			.await
			.context("Activation failed")?;

		let elapsed_ms = start.elapsed().as_millis() as u64;

		// Record activation
		let record = ActivationRecord {
			extension_id:extension_id.to_string(),
			events:extension_info.activation_events.clone(),
			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_secs())
				.unwrap_or(0),
			duration_ms:elapsed_ms,
			success:activation_result.success,
			error:None,
		};

		// Save timestamp for later use
		let activation_timestamp = record.timestamp;

		self.activation_history.write().await.push(record);

		// Update extension state
		self.extension_manager
			.update_state(extension_id, ExtensionState::Activated)
			.await?;

		// Register handler
		let mut handlers = self.event_handlers.write().await;
		handlers.insert(
			extension_id.to_string(),
			ActivationHandler {
				extension_id:extension_id.to_string(),
				events:activation_events,
				activation_function:"activate".to_string(),
				is_active:true,
				last_activation:Some(activation_timestamp),
			},
		);

		info!("Extension activated in {}ms: {}", elapsed_ms, extension_id);

		Ok(ActivationResult {
			extension_id:extension_id.to_string(),
			success:true,
			time_ms:elapsed_ms,
			error:None,
			contributes:extension_info.capabilities.clone(),
		})
	}

	/// Deactivate an extension
	#[instrument(skip(self, extension_id))]
	pub async fn deactivate(&self, extension_id:&str) -> Result<()> {
		info!("Deactivating extension: {}", extension_id);

		// Remove handler
		let mut handlers = self.event_handlers.write().await;
		if let Some(mut handler) = handlers.remove(extension_id) {
			handler.is_active = false;
		}

		// Update extension state
		self.extension_manager
			.update_state(extension_id, ExtensionState::Deactivated)
			.await?;

		info!("Extension deactivated: {}", extension_id);

		Ok(())
	}

	/// Trigger activation for certain events
	#[instrument(skip(self, event, _context))]
	pub async fn trigger_activation(&self, event:&str, _context:&ActivationContext) -> Result<Vec<ActivationResult>> {
		info!("Triggering activation for event: {}", event);

		let activation_event = ActivationEvent::from_str(event)?;
		let handlers = self.event_handlers.read().await;

		let mut results = Vec::new();

		for (extension_id, handler) in handlers.iter() {
			// Check if extension should activate on this event
			if handler.is_active {
				continue; // Already active
			}

			if self.should_activate(&activation_event, &handler.events) {
				debug!("Activating extension {} for event: {}", extension_id, event);
				match self.activate(extension_id).await {
					Ok(result) => results.push(result),
					Err(e) => {
						warn!("Failed to activate extension {} for event {}: {}", extension_id, event, e);
					},
				}
			}
		}

		Ok(results)
	}

	/// Check if extension should activate for given event
	fn should_activate(&self, activation_event:&ActivationEvent, events:&[ActivationEvent]) -> bool {
		events.iter().any(|e| {
			match (e, activation_event) {
				(ActivationEvent::Star, _) => true,
				(ActivationEvent::Custom(pattern), _) => {
					WildMatch::new(pattern).matches(activation_event.to_string().as_str())
				},
				_ => e == activation_event,
			}
		})
	}

	/// Perform actual activation (placeholder - would call extension's activate
	/// function)
	async fn perform_activation(&self, extension_id:&str, _context:&ActivationContext) -> Result<ActivationResult> {
		// In real implementation, this would:
		// 1. Call the extension's activate function
		// 2. Pass the activation context
		// 3. Wait for activation to complete
		// 4. Handle any errors

		debug!("Performing activation for extension: {}", extension_id);

		// Placeholder implementation
		Ok(ActivationResult {
			extension_id:extension_id.to_string(),
			success:true,
			time_ms:0,
			error:None,
			contributes:Vec::new(),
		})
	}

	/// Get activation history
	pub async fn get_activation_history(&self) -> Vec<ActivationRecord> { self.activation_history.read().await.clone() }

	/// Get activation history for a specific extension
	pub async fn get_activation_history_for_extension(&self, extension_id:&str) -> Vec<ActivationRecord> {
		self.activation_history
			.read()
			.await
			.iter()
			.filter(|r| r.extension_id == extension_id)
			.cloned()
			.collect()
	}
}

/// Simple wildcard matching for flexible activation events
struct WildMatch {
	pattern:String,
}

impl WildMatch {
	fn new(pattern:&str) -> Self { Self { pattern:pattern.to_lowercase() } }

	fn matches(&self, text:&str) -> bool {
		let text = text.to_lowercase();

		// Handle * wildcard
		if self.pattern == "*" {
			return true;
		}

		// Handle patterns starting with *
		if self.pattern.starts_with('*') {
			let suffix = &self.pattern[1..];
			return text.ends_with(suffix);
		}

		// Handle patterns ending with *
		if self.pattern.ends_with('*') {
			let prefix = &self.pattern[..self.pattern.len() - 1];
			return text.starts_with(prefix);
		}

		// Exact match
		self.pattern == text
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_activation_event_parsing() {
		let event = ActivationEvent::from_str("*").unwrap();
		assert_eq!(event, ActivationEvent::Star);

		let event = ActivationEvent::from_str("onCommand:test.command").unwrap();
		assert_eq!(event, ActivationEvent::Command("test.command".to_string()));

		let event = ActivationEvent::from_str("onLanguage:rust").unwrap();
		assert_eq!(event, ActivationEvent::Language("rust".to_string()));
	}

	#[test]
	fn test_activation_event_to_string() {
		assert_eq!(ActivationEvent::Star.to_string(), "*");
		assert_eq!(ActivationEvent::Command("test".to_string()).to_string(), "onCommand:test");
		assert_eq!(ActivationEvent::Language("rust".to_string()).to_string(), "onLanguage:rust");
	}

	#[test]
	fn test_activation_context_default() {
		let context = ActivationContext::default();
		assert!(context.workspace_path.is_none());
		assert!(context.current_file.is_none());
		assert!(!context.active_editor);
	}

	#[test]
	fn test_wildcard_matching() {
		let matcher = WildMatch::new("*");
		assert!(matcher.matches("anything"));

		let matcher = WildMatch::new("prefix*");
		assert!(matcher.matches("prefix_suffix"));
		assert!(!matcher.matches("noprefix_suffix"));

		let matcher = WildMatch::new("*suffix");
		assert!(matcher.matches("prefix_suffix"));
		assert!(!matcher.matches("prefix_suffix_not"));
	}
}
