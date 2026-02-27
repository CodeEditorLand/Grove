//! Extension Manager Module
//!
//! Handles extension discovery, loading, and management.
//! Provides query and monitoring capabilities for extensions.

use crate::Host::HostConfig;
use crate::WASM::Runtime::WASMRuntime;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Extension manager for handling extension lifecycle
pub struct ExtensionManagerImpl {
    /// WASM runtime for executing extensions
    #[allow(dead_code)]
    wasm_runtime: Arc<WASMRuntime>,
    /// Host configuration
    config: HostConfig,
    /// Loaded extensions
    extensions: Arc<RwLock<HashMap<String, ExtensionInfo>>>,
    /// Extension statistics
    stats: Arc<RwLock<ExtensionStats>>,
}

/// Extension information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    /// Extension ID (e.g., "publisher.extension-name")
    pub id: String,
    /// Extension display name
    pub display_name: String,
    /// Extension description
    pub description: String,
    /// Extension version
    pub version: String,
    /// Publisher name
    pub publisher: String,
    /// Path to extension directory
    pub path: PathBuf,
    /// Entry point file
    pub entry_point: PathBuf,
    /// Activation events
    pub activation_events: Vec<String>,
    /// Type of extension (wasm, native, etc.)
    pub extension_type: ExtensionType,
    /// Extension state
    pub state: ExtensionState,
    /// Extension capabilities
    pub capabilities: Vec<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Extension manifest (JSON)
    pub manifest: serde_json::Value,
    /// Load timestamp
    pub loaded_at: u64,
    /// Activation timestamp
    pub activated_at: Option<u64>,
}

/// Extension type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionType {
    /// WebAssembly extension
    WASM,
    /// Native Rust extension
    Native,
    /// JavaScript/TypeScript extension (via Cocoon compatibility)
    JavaScript,
    /// Unknown type
    Unknown,
}

/// Extension state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionState {
    /// Extension is loaded but not activated
    Loaded,
    /// Extension is activated and running
    Activated,
    /// Extension is deactivated
    Deactivated,
    /// Extension encountered an error
    Error,
}

/// Extension statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtensionStats {
    /// Total number of extensions loaded
    pub total_loaded: usize,
    /// Total number of extensions activated
    pub total_activated: usize,
    /// Total number of extensions deactivated
    pub total_deactivated: usize,
    /// Total activation time in milliseconds
    pub total_activation_time_ms: u64,
    /// Number of errors encountered
    pub errors: u64,
}

impl ExtensionManagerImpl {
    /// Create a new extension manager
    pub fn new(wasm_runtime: Arc<WASMRuntime>, config: HostConfig) -> Self {
        Self {
            wasm_runtime,
            config,
            extensions: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ExtensionStats::default())),
        }
    }

    /// Load an extension from a path
    #[instrument(skip(self, path))]
    pub async fn load_extension(&self, path: &PathBuf) -> Result<String> {
        info!("Loading extension from: {:?}", path);

        // Validate path
        if !path.exists() {
            return Err(anyhow::anyhow!("Extension path does not exist: {:?}", path));
        }

        // Parse manifest
        let manifest = self.parse_manifest(path)?;
        let extension_id = self.extract_extension_id(&manifest)?;

        // Check if extension is already loaded
        let extensions = self.extensions.read().await;
        if extensions.contains_key(&extension_id) {
            warn!("Extension already loaded: {}", extension_id);
            return Ok(extension_id);
        }
        drop(extensions);

        // Determine extension type
        let extension_type = self.determine_extension_type(path, &manifest)?;

        // Create extension info
        let extension_info = ExtensionInfo {
            id: extension_id.clone(),
            display_name: manifest
                .get("displayName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: manifest
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            version: manifest
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string(),
            publisher: manifest
                .get("publisher")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            path: path.clone(),
            entry_point: path.join(
                manifest
                    .get("main")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dist/extension.js"),
            ),
            activation_events: self.extract_activation_events(&manifest),
            extension_type,
            state: ExtensionState::Loaded,
            capabilities: self.extract_capabilities(&manifest),
            dependencies: self.extract_dependencies(&manifest),
            manifest,
            loaded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            activated_at: None,
        };

        // Register extension
        let mut extensions = self.extensions.write().await;
        extensions.insert(extension_id.clone(), extension_info);

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_loaded += 1;

        info!("Extension loaded successfully: {}", extension_id);

        Ok(extension_id)
    }

    /// Unload an extension
    #[instrument(skip(self, extension_id))]
    pub async fn unload_extension(&self, extension_id: &str) -> Result<()> {
        info!("Unloading extension: {}", extension_id);

        let mut extensions = self.extensions.write().await;
        extensions.remove(extension_id);

        info!("Extension unloaded: {}", extension_id);

        Ok(())
    }

    /// Get an extension by ID
    pub async fn get_extension(&self, extension_id: &str) -> Option<ExtensionInfo> {
        self.extensions.read().await.get(extension_id).cloned()
    }

    /// List all loaded extensions
    pub async fn list_extensions(&self) -> Vec<String> {
        self.extensions
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }

    /// List extensions in a specific state
    pub async fn list_extensions_by_state(&self, state: ExtensionState) -> Vec<ExtensionInfo> {
        self.extensions
            .read()
            .await
            .values()
            .filter(|ext| ext.state == state)
            .cloned()
            .collect()
    }

    /// Update extension state
    #[instrument(skip(self, extension_id))]
    pub async fn update_state(&self, extension_id: &str, state: ExtensionState) -> Result<()> {
        let mut extensions = self.extensions.write().await;
        if let Some(info) = extensions.get_mut(extension_id) {
            info.state = state;
            if state == ExtensionState::Activated {
                info.activated_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                );
                
                let mut stats = self.stats.write().await;
                stats.total_activated += 1;
            } else if state == ExtensionState::Deactivated {
                let mut stats = self.stats.write().await;
                stats.total_deactivated += 1;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Extension not found: {}", extension_id))
        }
    }

    /// Get extension manager statistics
    pub async fn stats(&self) -> ExtensionStats {
        self.stats.read().await.clone()
    }

    /// Discover extensions in configured paths
    #[instrument(skip(self))]
    pub async fn discover_extensions(&self) -> Result<Vec<PathBuf>> {
        info!("Discovering extensions in configured paths");

        let mut extensions = Vec::new();

        for discovery_path in &self.config.discovery_paths {
            match self.discover_in_path(discovery_path).await {
                Ok(mut found) => extensions.append(&mut found),
                Err(e) => {
                    warn!("Failed to discover extensions in {}: {}", discovery_path, e);
                }
            }
        }

        info!("Discovered {} extensions", extensions.len());

        Ok(extensions)
    }

    /// Discover extensions in a specific path
    async fn discover_in_path(&self, path: &str) -> Result<Vec<PathBuf>> {
        let path = PathBuf::from(shellexpand::tilde(path).as_ref());

        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut extensions = Vec::new();

        // Read directory entries
        let mut entries = tokio::fs::read_dir(&path)
            .await
            .context(format!("Failed to read directory: {:?}", path))?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            
            // Skip if not a directory
            if !entry_path.is_dir() {
                continue;
            }

            // Check for package.json or manifest.json
            let manifest_path = entry_path.join("package.json");
            let alt_manifest_path = entry_path.join("manifest.json");

            if manifest_path.exists() || alt_manifest_path.exists() {
                extensions.push(entry_path.clone());
                debug!("Discovered extension: {:?}", entry_path);
            }
        }

        Ok(extensions)
    }

    /// Parse extension manifest
    fn parse_manifest(&self, path: &Path) -> Result<serde_json::Value> {
        let manifest_path = path.join("package.json");
        let alt_manifest_path = path.join("manifest.json");

        let manifest_content = if manifest_path.exists() {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(tokio::fs::read_to_string(&manifest_path))
                .context("Failed to read package.json")?
        } else if alt_manifest_path.exists() {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(tokio::fs::read_to_string(&alt_manifest_path))
                .context("Failed to read manifest.json")?
        } else {
            return Err(anyhow::anyhow!("No manifest found in extension path"));
        };

        let manifest: serde_json::Value =
            serde_json::from_str(&manifest_content).context("Failed to parse manifest")?;

        Ok(manifest)
    }

    /// Extract extension ID from manifest
    fn extract_extension_id(&self, manifest: &serde_json::Value) -> Result<String> {
        let publisher = manifest
            .get("publisher")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing publisher in manifest"))?;

        let name = manifest
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing name in manifest"))?;

        Ok(format!("{}.{}", publisher, name))
    }

    /// Determine extension type
    fn determine_extension_type(
        &self,
        path: &Path,
        manifest: &serde_json::Value,
    ) -> Result<ExtensionType> {
        // Check for WASM file
        let wasm_path = path.join("extension.wasm");
        if wasm_path.exists() {
            return Ok(ExtensionType::WASM);
        }

        // Check for Rust project
        let cargo_path = path.join("Cargo.toml");
        if cargo_path.exists() {
            return Ok(ExtensionType::Native);
        }

        // Check for JavaScript/TypeScript
        let main = manifest.get("main").and_then(|v| v.as_str());
        if let Some(main) = main {
            let main_path = path.join(main);
            if main_path.exists() && (main.ends_with(".js") || main.ends_with(".ts")) {
                return Ok(ExtensionType::JavaScript);
            }
        }

        Ok(ExtensionType::Unknown)
    }

    /// Extract activation events from manifest
    fn extract_activation_events(&self, manifest: &serde_json::Value) -> Vec<String> {
        manifest
            .get("activationEvents")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract capabilities from manifest
    fn extract_capabilities(&self, manifest: &serde_json::Value) -> Vec<String> {
        manifest
            .get("capabilities")
            .and_then(|v| v.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Extract dependencies from manifest
    fn extract_dependencies(&self, manifest: &serde_json::Value) -> Vec<String> {
        manifest
            .get("extensionDependencies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_type() {
        assert_eq!(ExtensionType::WASM, ExtensionType::WASM);
        assert_eq!(ExtensionType::Native, ExtensionType::Native);
        assert_eq!(ExtensionType::JavaScript, ExtensionType::JavaScript);
    }

    #[test]
    fn test_extension_state() {
        assert_eq!(ExtensionState::Loaded, ExtensionState::Loaded);
        assert_eq!(ExtensionState::Activated, ExtensionState::Activated);
        assert_eq!(ExtensionState::Deactivated, ExtensionState::Deactivated);
        assert_eq!(ExtensionState::Error, ExtensionState::Error);
    }

    #[tokio::test]
    async fn test_extension_manager_creation() {
        let wasm_runtime = Arc::new(
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(crate::WASM::Runtime::WASMRuntime::new(
                    crate::WASM::Runtime::WASMConfig::default(),
                ))
                .unwrap()
        );
        let config = HostConfig::default();
        let manager = ExtensionManagerImpl::new(wasm_runtime, config);
        
        assert_eq!(manager.list_extensions().await.len(), 0);
    }

    #[test]
    fn test_extension_stats_default() {
        let stats = ExtensionStats::default();
        assert_eq!(stats.total_loaded, 0);
        assert_eq!(stats.total_activated, 0);
    }
}
