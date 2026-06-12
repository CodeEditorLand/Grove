//! Activation engine for managing extension activation

use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use tokio::sync::RwLock;

use crate::{
	Host::{
		Activation::{ActivationContext, ActivationEvent, ActivationHandler, ActivationRecord, WildMatch},
		ActivationResult,
		ExtensionManager::{ExtensionManagerImpl, ExtensionState, ExtensionType},
		HostConfig,
	},
	WASM::ModuleLoader::ModuleLoaderImpl,
	dev_log,
};

/// Activation engine for managing extension activation
pub struct ActivationEngine {
	/// Extension manager
	extension_manager:Arc<ExtensionManagerImpl>,

	/// Module loader - used for the WASM activation path
	module_loader:Arc<ModuleLoaderImpl>,

	/// Host configuration
	config:HostConfig,

	/// Event handlers mapping
	event_handlers:Arc<RwLock<HashMap<String, ActivationHandler::ActivationHandler>>>,

	/// Activation history
	activation_history:Arc<RwLock<Vec<ActivationRecord::ActivationRecord>>>,
}

impl ActivationEngine {
	/// Create a new activation engine
	pub fn new(extension_manager:Arc<ExtensionManagerImpl>, config:HostConfig) -> Self {
		use crate::WASM::{ModuleLoader::ModuleLoaderImpl, Runtime::WASMConfig};

		let module_loader = Arc::new(ModuleLoaderImpl::new(
			Arc::clone(extension_manager.wasm_runtime()),
			WASMConfig::default(),
		));

		Self {
			extension_manager,

			module_loader,

			config,

			event_handlers:Arc::new(RwLock::new(HashMap::new())),

			activation_history:Arc::new(RwLock::new(Vec::new())),
		}
	}

	/// Activate an extension
	pub async fn activate(&self, extension_id:&str) -> Result<ActivationResult> {
		dev_log!("extensions", "Activating extension: {}", extension_id);

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
				dev_log!("extensions", "warn: extension already active: {}", extension_id);

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
		let activation_events:Result<Vec<ActivationEvent::ActivationEvent>> = extension_info
			.activation_events
			.iter()
			.map(|e| ActivationEvent::ActivationEvent::from_str(e))
			.collect();

		let activation_events = activation_events.with_context(|| "Failed to parse activation events")?;

		// Create activation context
		let context = ActivationContext::ActivationContext::default();

		// Perform activation (in real implementation, this would call the extension's
		// activate function)
		let activation_result = self
			.perform_activation(extension_id, &context)
			.await
			.context("Activation failed")?;

		let elapsed_ms = start.elapsed().as_millis() as u64;

		// Record activation
		let record = ActivationRecord::ActivationRecord {
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
			ActivationHandler::ActivationHandler {
				extension_id:extension_id.to_string(),
				events:activation_events,
				activation_function:"activate".to_string(),
				is_active:true,
				last_activation:Some(activation_timestamp),
			},
		);

		dev_log!("extensions", "Extension activated in {}ms: {}", elapsed_ms, extension_id);

		Ok(ActivationResult {
			extension_id:extension_id.to_string(),
			success:true,
			time_ms:elapsed_ms,
			error:None,
			contributes:extension_info.capabilities.clone(),
		})
	}

	/// Deactivate an extension
	pub async fn deactivate(&self, extension_id:&str) -> Result<()> {
		dev_log!("extensions", "Deactivating extension: {}", extension_id);

		// Remove handler
		let mut handlers = self.event_handlers.write().await;

		if let Some(mut handler) = handlers.remove(extension_id) {
			handler.is_active = false;
		}

		// Update extension state
		self.extension_manager
			.update_state(extension_id, ExtensionState::Deactivated)
			.await?;

		dev_log!("extensions", "Extension deactivated: {}", extension_id);

		Ok(())
	}

	/// Trigger activation for certain events
	pub async fn trigger_activation(
		&self,
		event:&str,
		_context:&ActivationContext::ActivationContext,
	) -> Result<Vec<ActivationResult>> {
		dev_log!("extensions", "Triggering activation for event: {}", event);

		let activation_event = ActivationEvent::ActivationEvent::from_str(event)?;

		let handlers = self.event_handlers.read().await;

		let mut results = Vec::new();

		for (extension_id, handler) in handlers.iter() {
			// Check if extension should activate on this event
			if handler.is_active {
				continue; // Already active
			}

			if self.should_activate(&activation_event, &handler.events) {
				dev_log!("extensions", "Activating extension {} for event: {}", extension_id, event);

				match self.activate(extension_id).await {
					Ok(result) => results.push(result),

					Err(e) => {
						dev_log!(
							"extensions",
							"warn: failed to activate extension {} for event {}: {}",
							extension_id,
							event,
							e
						);
					},
				}
			}
		}

		Ok(results)
	}

	/// Check if extension should activate for given event
	fn should_activate(
		&self,
		activation_event:&ActivationEvent::ActivationEvent,
		events:&[ActivationEvent::ActivationEvent],
	) -> bool {
		events.iter().any(|e| {
			match (e, activation_event) {
				(ActivationEvent::ActivationEvent::Star, _) => true,
				(ActivationEvent::ActivationEvent::Custom(pattern), _) => {
					WildMatch::WildMatch::new(pattern).matches(activation_event.to_string().as_str())
				},
				_ => e == activation_event,
			}
		})
	}

	/// Perform actual activation - dispatches by extension type.
	/// WASM extensions are loaded via ModuleLoaderImpl and their exported
	/// `activate` function is called through a wasmtime typed func.
	/// Non-WASM extensions are deferred to the host (Cocoon/Node).
	async fn perform_activation(
		&self,
		extension_id:&str,
		_context:&ActivationContext::ActivationContext,
	) -> Result<ActivationResult> {
		let extension_info = match self.extension_manager.get_extension(extension_id).await {
			Some(info) => info,

			None => return Err(anyhow::anyhow!("Extension not found: {}", extension_id)),
		};

		match extension_info.extension_type {
			ExtensionType::WASM => {
				let wasm_module = self
					.module_loader
					.load_from_file(&extension_info.entry_point)
					.await
					.with_context(|| format!("Failed to load WASM for {}", extension_id))?;

				// Re-read bytes to compile a wasmtime::Module for instantiation
				let wasm_bytes = tokio::fs::read(&extension_info.entry_point).await?;

				let module = self.module_loader.runtime().compile_module(&wasm_bytes)?;

				let store = self.module_loader.runtime().create_store()?;

				let mut instance = self.module_loader.instantiate(&module, store).await?;

				if wasm_module.exported_functions.iter().any(|f| f == "activate") {
					let activate = instance
						.instance
						.get_typed_func::<(), ()>(&mut instance.store, "activate")
						.map_err(|e| anyhow::anyhow!("activate func error: {}", e))?;

					activate
						.call(&mut instance.store, ())
						.map_err(|e| anyhow::anyhow!("activate call failed: {}", e))?;

					dev_log!("extensions", "WASM activate() called for {}", extension_id);
				} else {
					dev_log!("extensions", "no activate export in WASM module for {}", extension_id);
				}

				Ok(ActivationResult {
					extension_id:extension_id.to_string(),
					success:true,
					time_ms:0,
					error:None,
					contributes:Vec::new(),
				})
			},

			ExtensionType::JavaScript | ExtensionType::Native | ExtensionType::Unknown => {
				dev_log!("extensions", "non-WASM extension activation deferred to host: {}", extension_id);

				Ok(ActivationResult {
					extension_id:extension_id.to_string(),
					success:true,
					time_ms:0,
					error:None,
					contributes:Vec::new(),
				})
			},
		}
	}

	/// Get activation history
	pub async fn get_activation_history(&self) -> Vec<ActivationRecord::ActivationRecord> {
		self.activation_history.read().await.clone()
	}

	/// Get activation history for a specific extension
	pub async fn get_activation_history_for_extension(
		&self,
		extension_id:&str,
	) -> Vec<ActivationRecord::ActivationRecord> {
		self.activation_history
			.read()
			.await
			.iter()
			.filter(|r| r.extension_id == extension_id)
			.cloned()
			.collect()
	}
}
