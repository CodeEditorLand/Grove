//! Configuration Service Module
//!
//! Provides configuration management for Grove.
//! Handles reading, writing, and watching configuration changes.

use crate::Services::Service;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

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
    pub value: Value,
    /// Scope
    pub scope: ConfigurationScope,
    /// Timestamp of last modification
    pub modified_at: u64,
}

/// Configuration service
pub struct ConfigurationService {
    /// Service name
    name: String,
    /// Configuration data
    config: Arc<RwLock<HashMap<String, ConfigurationValue>>>,
    /// Configuration paths
    config_paths: Arc<RwLock<HashMap<ConfigurationScope, PathBuf>>>,
    /// Running flag
    running: Arc<RwLock<bool>>,
    /// Watchers
    watchers: Arc<RwLock<HashMap<String, Vec<ConfigurationWatcherCallback>>>>,
}

/// Configuration watcher callback
type ConfigurationWatcherCallback = Arc<RwLock<dyn Fn(String, Value) -> Result<()> + Send + Sync>>;

impl ConfigurationService {
    /// Create a new configuration service
    pub fn new(config_path: Option<PathBuf>) -> Self {
        let mut config_paths = HashMap::new();
        
        if let Some(path) = config_path {
            config_paths.insert(ConfigurationScope::Global, path);
        }

        Self {
            name: "ConfigurationService".to_string(),
            config: Arc::new(RwLock::new(HashMap::new())),
            config_paths: Arc::new(RwLock::new(config_paths)),
            running: Arc::new(RwLock::new(false)),
            watchers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a configuration value
    #[instrument(skip(self))]
    pub async fn get(&self, key: &str) -> Option<Value> {
        debug!("Getting configuration value: {}", key);
        self.config.read().await.get(key).map(|v| v.value.clone())
    }

    /// Get a configuration value with a default
    pub async fn get_with_default(&self, key: &str, default: Value) -> Value {
        self.get(key).await.unwrap_or(default)
    }

    /// Set a configuration value
    #[instrument(skip(self, value))]
    pub async fn set(&self, key: String, value: Value, scope: ConfigurationScope) -> Result<()> {
        debug!("Setting configuration value: {} = {:?}", key, value);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let config_value = ConfigurationValue {
            value: value.clone(),
            scope,
            modified_at: now,
        };

        self.config.write().await.insert(key.clone(), config_value);

        // Notify watchers
        self.notify_watchers(key, value).await;

        Ok(())
    }

    /// Remove a configuration value
    #[instrument(skip(self))]
    pub async fn remove(&self, key: String) -> Result<bool> {
        debug!("Removing configuration value: {}", key);
        
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
    pub async fn get_all_in_scope(
        &self,
        scope: ConfigurationScope,
    ) -> HashMap<String, Value> {
        self.config
            .read()
            .await
            .iter()
            .filter(|(_, v)| v.scope == scope)
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    /// Load configuration from a file
    #[instrument(skip(self, path))]
    pub async fn load_from_file(&self, path: &Path, scope: ConfigurationScope) -> Result<()> {
        info!("Loading configuration from: {:?}", path);

        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read configuration file")?;

        let config: Value = serde_json::from_str(&content)
            .context("Failed to parse configuration file")?;

        self.load_from_value(config, scope).await?;

        // Store path for future reference
        self.config_paths
            .write()
            .await
            .insert(scope, path.to_path_buf());

        info!("Configuration loaded successfully");

        Ok(())
    }

    /// Load configuration from a value
    #[instrument(skip(self, value))]
    pub async fn load_from_value(&self, value: Value, scope: ConfigurationScope) -> Result<()> {
        if let Value::Object(object) = value {
            let mut config = self.config.write().await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            for (key, val) in object {
                config.insert(
                    key,
                    ConfigurationValue {
                        value: val,
                        scope,
                        modified_at: now,
                    },
                );
            }
        }

        Ok(())
    }

    /// Save configuration to a file
    #[instrument(skip(self, path))]
    pub async fn save_to_file(&self, path: &Path, scope: ConfigurationScope) -> Result<()> {
        info!("Saving configuration to: {:?}", path);

        let config = self.get_all_in_scope(scope).await;
        let config_value = Value::Object(
            config.into_iter().map(|(k, v)| (k, v)).collect()
        );

        let content = serde_json::to_string_pretty(&config_value)
            .context("Failed to serialize configuration")?;

        tokio::fs::write(path, content)
            .await
            .context("Failed to write configuration file")?;

        info!("Configuration saved successfully");

        Ok(())
    }

    /// Register a configuration watcher
    #[instrument(skip(self, key, callback))]
    pub async fn register_watcher<F>(&self, key: String, callback: F)
    where
        F: Fn(String, Value) -> Result<()> + Send + Sync + 'static,
    {
        let mut watchers = self.watchers.write().await;
        watchers
            .entry(key)
            .or_insert_with(Vec::new)
            .push(Arc::new(RwLock::new(callback)));
        debug!("Registered configuration watcher for: {}", key);
    }

    /// Unregister a configuration watcher
    #[instrument(skip(self))]
    pub async fn unregister_watcher(&self, key: String) -> Result<bool> {
        let mut watchers = self.watchers.write().await;
        let removed = watchers.remove(&key).is_some();
        Ok(removed)
    }

    /// Notify watchers of configuration changes
    async fn notify_watchers(&self, key: String, value: Value) {
        let watchers = self.watchers.read().await;
        
        if let Some(callbacks) = watchers.get(&key) {
            for callback in callbacks {
                if let Err(e) = callback.read().await(key.clone(), value.clone()) {
                    warn!("Configuration watcher callback failed: {}", e);
                }
            }
        }
    }

    /// Get configuration paths
    pub async fn get_config_paths(&self) -> HashMap<ConfigurationScope, PathBuf> {
        self.config_paths.read().await.clone()
    }
}

impl Service for ConfigurationService {
    fn name(&self) -> &str {
        &self.name
    }

    async fn start(&self) -> Result<()> {
        info!("Starting configuration service");
        
        *self.running.write().await = true;
        
        info!("Configuration service started");
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        info!("Stopping configuration service");
        
        *self.running.write().await = false;
        
        info!("Configuration service stopped");
        Ok(())
    }

    async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_configuration_service_basic() {
        let service = ConfigurationService::new(None);
        service.start().await.unwrap();
        
        // Test setting and getting
        service.set(
            "test.key".to_string(),
            serde_json::json!("test-value"),
            ConfigurationScope::Global,
        )
        .await
        .unwrap();

        let value = service.get("test.key").await;
        assert_eq!(value, Some(serde_json::json!("test-value")));

        service.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_with_default() {
        let service = ConfigurationService::new(None);
        
        let default = serde_json::json!("default-value");
        let value = service.get_with_default("nonexistent.key", default.clone()).await;
        assert_eq!(value, default);
    }

    #[tokio::test]
    async fn test_get_all_in_scope() {
        let service = ConfigurationService::new(None);
        
        service.set(
            "key1".to_string(),
            serde_json::json!("value1"),
            ConfigurationScope::Global,
        )
        .await
        .unwrap();

        service.set(
            "key2".to_string(),
            serde_json::json!("value2"),
            ConfigurationScope::Workspace,
        )
        .await
        .unwrap();

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
