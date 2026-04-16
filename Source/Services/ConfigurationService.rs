//! Configuration Service Module
//!
//! Provides configuration management for Grove.
//! Handles reading, writing, and watching configuration changes.

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::{Context, Result};
use serde_json::Value;
use tokio::sync::RwLock;
use crate::dev_log;

use crate::Services::Service;

/// Configuration scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationScope {
	/// Global configuration
	Global,
	/// Workspace configuration
	Workspace,
	/// Extension-specific configuration
	Extension,
}

/// Configuration value
#[derive(Debug, Clone)]
pub struct ConfigurationValue {
	/// Value
	pub value:Value,
	/// Scope
	pub scope:ConfigurationScope,
	/// Timestamp of last modification
	pub modified_at:u64,
}

/// Configuration service
pub struct ConfigurationServiceImpl {
	/// Service name
	name:String,
	/// Configuration data
	config:Arc<RwLock<HashMap<String, ConfigurationValue>>>,
	/// Configuration paths
	config_paths:Arc<RwLock<HashMap<ConfigurationScope, PathBuf>>>,
	/// Running flag
	running:Arc<RwLock<bool>>,
	/// Watchers
	watchers:Arc<RwLock<HashMap<String, Vec<ConfigurationWatcherCallback>>>>,
}

/// Configuration watcher callback
type ConfigurationWatcherCallback = Arc<RwLock<dyn Fn(String, Value) -> Result<()> + Send + Sync>>;

impl ConfigurationServiceImpl {
	/// Create a new configuration service
	pub fn new(config_path:Option<PathBuf>) -> Self {
		let mut config_paths = HashMap::new();

		if let Some(path) = config_path {
			config_paths.insert(ConfigurationScope::Global, path);
		}

		Self {
			name:"ConfigurationService".to_string(),
			config:Arc::new(RwLock::new(HashMap::new())),
			config_paths:Arc::new(RwLock::new(config_paths)),
			running:Arc::new(RwLock::new(false)),
			watchers:Arc::new(RwLock::new(HashMap::new())),
		}
	}

	/// Get a configuration value
	pub async fn get(&self, key:&str) -> Option<Value> {
		dev_log!("config", "Getting configuration value: {}", key);
		self.config.read().await.get(key).map(|v| v.value.clone())
	}

	/// Get a configuration value with a default
	pub async fn get_with_default(&self, key:&str, default:Value) -> Value { self.get(key).await.unwrap_or(default) }

	/// Set a configuration value
	pub async fn set(&self, key:String, value:Value, scope:ConfigurationScope) -> Result<()> {
		dev_log!("config", "Setting configuration value: {} = {:?}", key, value);

		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_secs())
			.unwrap_or(0);

		let config_value = ConfigurationValue { value:value.clone(), scope, modified_at:now };

		self.config.write().await.insert(key.clone(), config_value);

		// Notify watchers
		self.notify_watchers(key, value).await;

		Ok(())
	}

	/// Remove a configuration value
	pub async fn remove(&self, key:String) -> Result<bool> {
		dev_log!("config", "Removing configuration value: {}", key);

		let removed = self.config.write().await.remove(&key).is_some();
		Ok(removed)
	}

	/// Get all configuration values
	pub async fn get_all(&self) -> HashMap<String, Value> {
		self.config
			.read()
			.await
			.iter()
			.map(|(k, v)| (k.clone(), v.value.clone()))
			.collect()
	}

	/// Get all configuration values in a scope
	pub async fn get_all_in_scope(&self, scope:ConfigurationScope) -> HashMap<String, Value> {
		self.config
			.read()
			.await
			.iter()
			.filter(|(_, v)| v.scope == scope)
			.map(|(k, v)| (k.clone(), v.value.clone()))
			.collect()
	}

	/// Load configuration from a file
	pub async fn load_from_file(&self, path:&Path, scope:ConfigurationScope) -> Result<()> {
		dev_log!("config", "Loading configuration from: {:?}", path);

		let content = tokio::fs::read_to_string(path)
			.await
			.context("Failed to read configuration file")?;

		let config:Value = serde_json::from_str(&content).context("Failed to parse configuration file")?;

		self.load_from_value(config, scope).await?;

		// Store path for future reference
		self.config_paths.write().await.insert(scope, path.to_path_buf());

		dev_log!("config", "Configuration loaded successfully");

		Ok(())
	}

	/// Load configuration from a value
	pub async fn load_from_value(&self, value:Value, scope:ConfigurationScope) -> Result<()> {
		if let Value::Object(object) = value {
			let mut config = self.config.write().await;
			let now = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_secs())
				.unwrap_or(0);

			for (key, val) in object {
				config.insert(key, ConfigurationValue { value:val, scope, modified_at:now });
			}
		}

		Ok(())
	}

	/// Save configuration to a file
	pub async fn save_to_file(&self, path:&Path, scope:ConfigurationScope) -> Result<()> {
		dev_log!("config", "Saving configuration to: {:?}", path);

		let config = self.get_all_in_scope(scope).await;
		let config_value = Value::Object(config.into_iter().map(|(k, v)| (k, v)).collect());

		let content = serde_json::to_string_pretty(&config_value).context("Failed to serialize configuration")?;

		tokio::fs::write(path, content)
			.await
			.context("Failed to write configuration file")?;

		dev_log!("config", "Configuration saved successfully");

		Ok(())
	}

	/// Register a configuration watcher
	pub async fn register_watcher<F>(&self, key:String, callback:F)
	where
		F: Fn(String, Value) -> Result<()> + Send + Sync + 'static, {
		let key_clone = key.clone();
		let mut watchers = self.watchers.write().await;
		watchers
			.entry(key)
			.or_insert_with(Vec::new)
			.push(Arc::new(RwLock::new(callback)));
		dev_log!("config", "Registered configuration watcher for: {}", key_clone);
	}

	/// Unregister a configuration watcher
	pub async fn unregister_watcher(&self, key:String) -> Result<bool> {
		let mut watchers = self.watchers.write().await;
		let removed = watchers.remove(&key).is_some();
		Ok(removed)
	}

	/// Notify watchers of configuration changes
	async fn notify_watchers(&self, key:String, value:Value) {
		let watchers = self.watchers.read().await;

		if let Some(callbacks) = watchers.get(&key) {
			for callback in callbacks {
				if let Err(e) = callback.read().await(key.clone(), value.clone()) {
					dev_log!("config", "warn: configuration watcher callback failed: {}", e);
				}
			}
		}
	}

	/// Get configuration paths
	pub async fn get_config_paths(&self) -> HashMap<ConfigurationScope, PathBuf> {
		self.config_paths.read().await.clone()
	}
}

impl Service for ConfigurationServiceImpl {
	fn name(&self) -> &str { &self.name }

	async fn start(&self) -> Result<()> {
		dev_log!("config", "Starting configuration service");

		*self.running.write().await = true;

		dev_log!("config", "Configuration service started");
		Ok(())
	}

	async fn stop(&self) -> Result<()> {
		dev_log!("config", "Stopping configuration service");

		*self.running.write().await = false;

		dev_log!("config", "Configuration service stopped");
		Ok(())
	}

	async fn is_running(&self) -> bool { *self.running.read().await }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_configuration_service_basic() {
		let service = ConfigurationServiceImpl::new(None);
		let _:anyhow::Result<()> = service.start().await;

		// Test setting and getting
		let _:anyhow::Result<()> = service
			.set(
				"test.key".to_string(),
				serde_json::json!("test-value"),
				ConfigurationScope::Global,
			)
			.await;

		let value = service.get("test.key").await;
		assert_eq!(value, Some(serde_json::json!("test-value")));

		let _:anyhow::Result<()> = service.stop().await;
	}

	#[tokio::test]
	async fn test_get_with_default() {
		let service = ConfigurationServiceImpl::new(None);

		let default = serde_json::json!("default-value");
		let value = service.get_with_default("nonexistent.key", default.clone()).await;
		assert_eq!(value, default);
	}

	#[tokio::test]
	async fn test_get_all_in_scope() {
		let service = ConfigurationServiceImpl::new(None);

		let _:anyhow::Result<()> = service
			.set("key1".to_string(), serde_json::json!("value1"), ConfigurationScope::Global)
			.await;

		let _:anyhow::Result<()> = service
			.set("key2".to_string(), serde_json::json!("value2"), ConfigurationScope::Workspace)
			.await;

		let global_values = service.get_all_in_scope(ConfigurationScope::Global).await;
		assert_eq!(global_values.len(), 1);
		assert_eq!(global_values.get("key1"), Some(&serde_json::json!("value1")));
	}

	#[test]
	fn test_configuration_scope() {
		let global = ConfigurationScope::Global;
		let workspace = ConfigurationScope::Workspace;
		let extension = ConfigurationScope::Extension;

		assert_eq!(global, ConfigurationScope::Global);
		assert_ne!(global, workspace);
		assert_ne!(global, extension);
	}
}
