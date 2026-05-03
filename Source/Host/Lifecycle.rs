//! Lifecycle Management Module
//!
//! Handles extension lifecycle events such as initialization,
//! shutdown, and state transitions.

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::dev_log;

/// Lifecycle event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleEvent {
	/// Extension is being initialized
	Initialize,
	/// Extension is being started
	Start,
	/// Extension is being stopped
	Stop,
	/// Extension is being disposed
	Dispose,
	/// Extension is reloading (hot reload)
	Reload,
	/// Extension is being suspended
	Suspend,
	/// Extension is being resumed
	Resume,
	/// Custom lifecycle event
	Custom(String),
}

/// Lifecycle state for extensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
	/// Extension has been created but not initialized
	Created,
	/// Extension is being initialized
	Initializing,
	/// Extension is active and running
	Running,
	/// Extension is being suspended
	Suspending,
	/// Extension is suspended
	Suspended,
	/// Extension is being stopped
	Stopping,
	/// Extension has been stopped
	Stopped,
	/// Extension is being disposed
	Disposing,
	/// Extension has been disposed
	Disposed,
	/// Extension is in an error state
	Error,
}

/// Lifecycle event handler callback
#[allow(dead_code)]
type LifecycleEventHandler = fn(&str, LifecycleEvent) -> Result<()>;

/// Lifecycle manager for extension lifecycle
pub struct LifecycleManager {
	/// Event handlers
	handlers:Arc<RwLock<HashMap<String, LifecycleHandlerInfo>>>,
	/// Extension states
	states:Arc<RwLock<HashMap<String, LifecycleState>>>,
	/// Event history
	event_history:Arc<RwLock<Vec<LifecycleEventRecord>>>,
}

/// Information about a lifecycle handler
#[derive(Debug, Clone)]
struct LifecycleHandlerInfo {
	/// Extension ID
	#[allow(dead_code)]
	extension_id:String,
	/// Current state
	state:LifecycleState,
	/// Supported events
	#[allow(dead_code)]
	supported_events:Vec<LifecycleEvent>,
	/// Last state change timestamp
	last_state_change:Option<u64>,
}

/// Record of a lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEventRecord {
	/// Extension ID
	pub extension_id:String,
	/// Event that occurred
	pub event:LifecycleEvent,
	/// Previous state
	pub previous_state:LifecycleState,
	/// New state
	pub new_state:LifecycleState,
	/// Timestamp
	pub timestamp:u64,
	/// Duration in milliseconds
	pub duration_ms:u64,
	/// Success flag
	pub success:bool,
	/// Error message (if failed)
	pub error:Option<String>,
}

impl LifecycleManager {
	/// Create a new lifecycle manager
	pub fn new() -> Self {
		Self {
			handlers:Arc::new(RwLock::new(HashMap::new())),
			states:Arc::new(RwLock::new(HashMap::new())),
			event_history:Arc::new(RwLock::new(Vec::new())),
		}
	}

	/// Register an extension for lifecycle management

	pub async fn register_extension(&self, extension_id:&str, initial_state:LifecycleState) -> Result<()> {
		dev_log!("extensions", "Registering extension for lifecycle management: {}", extension_id);

		let mut handlers = self.handlers.write().await;
		handlers.insert(
			extension_id.to_string(),
			LifecycleHandlerInfo {
				extension_id:extension_id.to_string(),
				state:initial_state,
				supported_events:vec![
					LifecycleEvent::Initialize,
					LifecycleEvent::Start,
					LifecycleEvent::Stop,
					LifecycleEvent::Dispose,
				],
				last_state_change:Some(
					std::time::SystemTime::now()
						.duration_since(std::time::UNIX_EPOCH)
						.map(|d| d.as_secs())
						.unwrap_or(0),
				),
			},
		);

		let mut states = self.states.write().await;
		states.insert(extension_id.to_string(), initial_state);

		dev_log!("extensions", "Extension registered: {}", extension_id);

		Ok(())
	}

	/// Unregister an extension from lifecycle management

	pub async fn unregister_extension(&self, extension_id:&str) -> Result<()> {
		dev_log!(
			"extensions",
			"Unregistering extension from lifecycle management: {}",
			extension_id
		);

		let mut handlers = self.handlers.write().await;
		handlers.remove(extension_id);

		let mut states = self.states.write().await;
		states.remove(extension_id);

		dev_log!("extensions", "Extension unregistered: {}", extension_id);

		Ok(())
	}

	/// Get the current state of an extension
	pub async fn get_state(&self, extension_id:&str) -> Option<LifecycleState> {
		self.states.read().await.get(extension_id).copied()
	}

	/// Transition an extension to a new state

	pub async fn transition(&self, extension_id:&str, event:LifecycleEvent) -> Result<LifecycleState> {
		dev_log!("lifecycle", "Transitioning extension {} with event: {:?}", extension_id, event);

		let start = std::time::Instant::now();

		// Get current state
		let current_state = self
			.get_state(extension_id)
			.await
			.ok_or_else(|| anyhow::anyhow!("Extension not found: {}", extension_id))?;

		// Clone event for later use before moving it
		let event_clone = event.clone();

		// Determine new state based on event
		let new_state = self.determine_next_state(current_state, event)?;

		// Perform state transition (in real implementation, this would call extension)
		self.perform_state_transition(extension_id, event_clone.clone(), new_state)
			.await?;

		let elapsed_ms = start.elapsed().as_millis() as u64;

		// Record event
		let record = LifecycleEventRecord {
			extension_id:extension_id.to_string(),
			event:event_clone,
			previous_state:current_state,
			new_state,
			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_secs())
				.unwrap_or(0),
			duration_ms:elapsed_ms,
			success:true,
			error:None,
		};

		self.event_history.write().await.push(record);

		dev_log!(
			"lifecycle",
			"Extension {} transitioned from {:?} to {:?} in {}ms",
			extension_id,
			current_state,
			new_state,
			elapsed_ms
		);

		Ok(new_state)
	}

	/// Determine the next state based on current state and event
	fn determine_next_state(&self, current_state:LifecycleState, event:LifecycleEvent) -> Result<LifecycleState> {
		match (current_state, event.clone()) {
			(LifecycleState::Created, LifecycleEvent::Initialize) => Ok(LifecycleState::Initializing),
			(LifecycleState::Initializing, LifecycleEvent::Start) => Ok(LifecycleState::Running),
			(LifecycleState::Running, LifecycleEvent::Suspend) => Ok(LifecycleState::Suspending),
			(LifecycleState::Suspending, _) => Ok(LifecycleState::Suspended),
			(LifecycleState::Suspended, LifecycleEvent::Resume) => Ok(LifecycleState::Running),
			(LifecycleState::Running, LifecycleEvent::Stop) => Ok(LifecycleState::Stopping),
			(LifecycleState::Stopping, _) => Ok(LifecycleState::Stopped),
			(LifecycleState::Stopped | LifecycleState::Suspended, LifecycleEvent::Dispose) => {
				Ok(LifecycleState::Disposing)
			},
			(LifecycleState::Disposing, _) => Ok(LifecycleState::Disposed),
			(LifecycleState::Running, LifecycleEvent::Reload) => Ok(LifecycleState::Running),
			_ => {
				Err(anyhow::anyhow!(
					"Invalid transition from {:?} with event {:?}",
					current_state,
					event
				))
			},
		}
	}

	/// Perform actual state transition
	async fn perform_state_transition(
		&self,
		extension_id:&str,
		event:LifecycleEvent,
		new_state:LifecycleState,
	) -> Result<()> {
		// In real implementation, this would:
		// 1. Call the extension's lifecycle handler
		// 2. Handle any errors
		// 3. Rollback on failure

		dev_log!(
			"lifecycle",
			"Performing state transition for extension {}: {:?} -> {:?}",
			extension_id,
			event,
			new_state
		);

		// Update state
		let mut handlers = self.handlers.write().await;
		if let Some(handler) = handlers.get_mut(extension_id) {
			handler.state = new_state;
			handler.last_state_change = Some(
				std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)
					.map(|d| d.as_secs())
					.unwrap_or(0),
			);
		}

		let mut states = self.states.write().await;
		states.insert(extension_id.to_string(), new_state);

		Ok(())
	}

	/// Trigger a lifecycle event for an extension

	pub async fn trigger_event(&self, extension_id:&str, event:LifecycleEvent) -> Result<()> {
		dev_log!("lifecycle", "Triggering lifecycle event for {}: {:?}", extension_id, event);

		self.transition(extension_id, event).await?;

		Ok(())
	}

	/// Get event history
	pub async fn get_event_history(&self) -> Vec<LifecycleEventRecord> { self.event_history.read().await.clone() }

	/// Get event history for a specific extension
	pub async fn get_event_history_for_extension(&self, extension_id:&str) -> Vec<LifecycleEventRecord> {
		self.event_history
			.read()
			.await
			.iter()
			.filter(|r| r.extension_id == extension_id)
			.cloned()
			.collect()
	}

	/// Get all registered extensions
	pub async fn get_registered_extensions(&self) -> Vec<String> {
		self.handlers.read().await.keys().cloned().collect()
	}

	/// Get extensions in a specific state
	pub async fn get_extensions_in_state(&self, state:LifecycleState) -> Vec<String> {
		self.states
			.read()
			.await
			.iter()
			.filter(|(_, s)| *s == &state)
			.map(|(id, _)| id.clone())
			.collect()
	}
}

impl Default for LifecycleManager {
	fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_lifecycle_state() {
		assert_eq!(LifecycleState::Created, LifecycleState::Created);
		assert_eq!(LifecycleState::Running, LifecycleState::Running);
		assert_ne!(LifecycleState::Created, LifecycleState::Running);
	}

	#[test]
	fn test_lifecycle_event() {
		assert_eq!(LifecycleEvent::Initialize, LifecycleEvent::Initialize);
		assert_eq!(
			LifecycleEvent::Custom("test".to_string()),
			LifecycleEvent::Custom("test".to_string())
		);
	}

	#[tokio::test]
	async fn test_lifecycle_manager_registration() {
		let manager = LifecycleManager::new();
		let result = manager.register_extension("test.ext", LifecycleState::Created).await;

		assert!(result.is_ok());
		assert_eq!(manager.get_state("test.ext").await, Some(LifecycleState::Created));
	}

	#[tokio::test]
	async fn test_state_transitions() {
		let manager = LifecycleManager::new();
		manager.register_extension("test.ext", LifecycleState::Created).await.unwrap();

		// Initialize
		let state = manager.transition("test.ext", LifecycleEvent::Initialize).await.unwrap();
		assert_eq!(state, LifecycleState::Initializing);

		// Start
		let state = manager.transition("test.ext", LifecycleEvent::Start).await.unwrap();
		assert_eq!(state, LifecycleState::Running);

		// Stop
		let state = manager.transition("test.ext", LifecycleEvent::Stop).await.unwrap();
		assert_eq!(state, LifecycleState::Stopping);
	}
}
