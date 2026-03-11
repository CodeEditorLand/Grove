//! WASM Module Loader
//!
//! Handles loading, compiling, and instantiating WebAssembly modules.
//! Provides utilities for working with WASM modules from various sources.

use std::{
	fs,
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};
use wasmtime::{Instance, Linker, Module, Store, StoreLimits};

use crate::WASM::Runtime::{WASMConfig, WASMRuntime};

/// WASM module wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WASMModule {
	/// Unique module identifier
	pub id:String,
	/// Module name (if available from name section)
	pub name:Option<String>,
	/// Path to the module file (if loaded from disk)
	pub path:Option<PathBuf>,
	/// Module source type
	pub source_type:ModuleSourceType,
	/// Module size in bytes
	pub size:usize,
	/// Exported functions
	pub exported_functions:Vec<String>,
	/// Exported memories
	pub exported_memories:Vec<String>,
	/// Exported tables
	pub exported_tables:Vec<String>,
	/// Import declarations
	pub imports:Vec<ImportDeclaration>,
	/// Compilation timestamp
	pub compiled_at:u64,
	/// Module hash (for caching)
	pub hash:Option<String>,
}

/// Source type of a WASM module
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleSourceType {
	/// Module loaded from a file
	File,
	/// Module loaded from in-memory bytes
	Memory,
	/// Module loaded from a network URL
	Url,
	/// Module generated dynamically
	Generated,
}

/// Import declaration for a WASM module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDeclaration {
	/// Module name being imported from
	pub module:String,
	/// Name of the imported item
	pub name:String,
	/// Kind of import
	pub kind:ImportKind,
}

/// Kind of import
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImportKind {
	/// Function import
	Function,
	/// Table import
	Table,
	/// Memory import
	Memory,
	/// Global import
	Global,
	/// Tag import
	Tag,
}

/// Module loading options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleLoadOptions {
	/// Enable lazy compilation
	pub lazy_compilation:bool,
	/// Enable module caching
	pub enable_cache:bool,
	/// Cache directory path
	pub cache_dir:Option<PathBuf>,
	/// Custom linker configuration
	pub custom_linker:bool,
	/// Validate module before loading
	pub validate:bool,
	/// Optimized compilation
	pub optimized:bool,
}

impl Default for ModuleLoadOptions {
	fn default() -> Self {
		Self {
			lazy_compilation:false,
			enable_cache:true,
			cache_dir:None,
			custom_linker:false,
			validate:true,
			optimized:true,
		}
	}
}

/// Module instance with store
pub struct WASMInstance {
	/// The WASM instance
	pub instance:Instance,
	/// The associated store
	pub store:Store<StoreLimits>,
	/// Instance ID
	pub id:String,
	/// Module reference
	pub module:Arc<Module>,
}

/// WASM Module Loader
pub struct ModuleLoaderImpl {
	runtime:Arc<WASMRuntime>,
	#[allow(dead_code)]
	config:WASMConfig,
	#[allow(dead_code)]
	linkers:Arc<RwLock<Vec<Linker<()>>>>,
	loaded_modules:Arc<RwLock<Vec<WASMModule>>>,
}

impl ModuleLoaderImpl {
	/// Create a new module loader
	pub fn new(runtime:Arc<WASMRuntime>, config:WASMConfig) -> Self {
		Self {
			runtime,
			config,
			linkers:Arc::new(RwLock::new(Vec::new())),
			loaded_modules:Arc::new(RwLock::new(Vec::new())),
		}
	}

	/// Load a WASM module from a file
	#[instrument(skip(self, path))]
	pub async fn load_from_file(&self, path:&Path) -> Result<WASMModule> {
		info!("Loading WASM module from file: {:?}", path);

		let wasm_bytes = fs::read(path).context(format!("Failed to read WASM file: {:?}", path))?;

		self.load_from_memory(&wasm_bytes, ModuleSourceType::File)
			.await
			.map(|mut module| {
				module.path = Some(path.to_path_buf());
				module
			})
	}

	/// Load a WASM module from memory
	#[instrument(skip(self, wasm_bytes))]
	pub async fn load_from_memory(&self, wasm_bytes:&[u8], source_type:ModuleSourceType) -> Result<WASMModule> {
		info!("Loading WASM module from memory ({} bytes)", wasm_bytes.len());

		// Validate if option is set
		if ModuleLoadOptions::default().validate {
			if !self.runtime.validate_module(wasm_bytes)? {
				return Err(anyhow::anyhow!("WASM module validation failed"));
			}
		}

		// Compile the module
		let module = self.runtime.compile_module(wasm_bytes)?;

		// Extract module information
		let module_info = self.extract_module_info(&module);

		// Create module wrapper
		let wasm_module = WASMModule {
			id:generate_module_id(&module_info.name),
			name:module_info.name,
			path:None,
			source_type,
			size:wasm_bytes.len(),
			exported_functions:module_info.exports.functions,
			exported_memories:module_info.exports.memories,
			exported_tables:module_info.exports.tables,
			imports:module_info.imports,
			compiled_at:std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
			hash:self.compute_hash(wasm_bytes),
		};

		// Store the module
		let mut loaded = self.loaded_modules.write().await;
		loaded.push(wasm_module.clone());

		debug!("WASM module loaded successfully: {}", wasm_module.id);

		Ok(wasm_module)
	}

	/// Load a WASM module from a URL
	#[instrument(skip(self, url))]
	pub async fn load_from_url(&self, url:&str) -> Result<WASMModule> {
		info!("Loading WASM module from URL: {}", url);

		// Fetch the module
		let response = reqwest::get(url)
			.await
			.context(format!("Failed to fetch WASM module from: {}", url))?;

		if !response.status().is_success() {
			return Err(anyhow::anyhow!("Failed to fetch WASM module: HTTP {}", response.status()));
		}

		let wasm_bytes = response.bytes().await?;

		self.load_from_memory(&wasm_bytes, ModuleSourceType::Url).await
	}

	/// Instantiate a loaded module
	#[instrument(skip(self, module))]
	pub async fn instantiate(&self, module:&Module, mut store:Store<StoreLimits>) -> Result<WASMInstance> {
		debug!("Instantiating WASM module");

		// Create linker with StoreLimits type
		let linker = self.runtime.create_linker::<StoreLimits>(true)?;

		// Instantiate
		let instance = linker
			.instantiate(&mut store, module)
			.map_err(|e| anyhow::anyhow!("Failed to instantiate WASM module: {}", e))?;

		let instance_id = generate_instance_id();

		debug!("WASM module instantiated: {}", instance_id);

		Ok(WASMInstance { instance, store, id:instance_id, module:Arc::new(module.clone()) })
	}

	/// Get all loaded modules
	pub async fn get_loaded_modules(&self) -> Vec<WASMModule> { self.loaded_modules.read().await.clone() }

	/// Get a loaded module by ID
	pub async fn get_module_by_id(&self, id:&str) -> Option<WASMModule> {
		let loaded = self.loaded_modules.read().await;
		loaded.iter().find(|m| m.id == id).cloned()
	}

	/// Unload a module
	pub async fn unload_module(&self, id:&str) -> Result<bool> {
		let mut loaded = self.loaded_modules.write().await;
		let pos = loaded.iter().position(|m| m.id == id);

		if let Some(pos) = pos {
			loaded.remove(pos);
			info!("WASM module unloaded: {}", id);
			Ok(true)
		} else {
			Ok(false)
		}
	}

	/// Extract module information from a compiled module
	fn extract_module_info(&self, module:&Module) -> ModuleInfo {
		let mut exports = Exports { functions:Vec::new(), memories:Vec::new(), tables:Vec::new(), globals:Vec::new() };

		let mut imports = Vec::new();

		for export in module.exports() {
			match export.ty() {
				wasmtime::ExternType::Func(_) => exports.functions.push(export.name().to_string()),
				wasmtime::ExternType::Memory(_) => exports.memories.push(export.name().to_string()),
				wasmtime::ExternType::Table(_) => exports.tables.push(export.name().to_string()),
				wasmtime::ExternType::Global(_) => exports.globals.push(export.name().to_string()),
				_ => {},
			}
		}

		for import in module.imports() {
			let kind = match import.ty() {
				wasmtime::ExternType::Func(_) => ImportKind::Function,
				wasmtime::ExternType::Memory(_) => ImportKind::Memory,
				wasmtime::ExternType::Table(_) => ImportKind::Table,
				wasmtime::ExternType::Global(_) => ImportKind::Global,
				_ => ImportKind::Tag,
			};
			imports.push(ImportDeclaration {
				module:import.module().to_string(),
				name:import.name().to_string(),
				kind,
			});
		}

		ModuleInfo {
			name:None, // Would need to parse name section
			exports,
			imports,
		}
	}

	/// Compute a hash of the WASM bytes for caching
	fn compute_hash(&self, wasm_bytes:&[u8]) -> Option<String> {
		use std::{
			collections::hash_map::DefaultHasher,
			hash::{Hash, Hasher},
		};

		let mut hasher = DefaultHasher::new();
		wasm_bytes.hash(&mut hasher);
		Some(format!("{:x}", hasher.finish()))
	}
}

// Helper structures and functions

struct ModuleInfo {
	name:Option<String>,
	exports:Exports,
	imports:Vec<ImportDeclaration>,
}

struct Exports {
	functions:Vec<String>,
	memories:Vec<String>,
	tables:Vec<String>,
	globals:Vec<String>,
}

fn generate_module_id(name:&Option<String>) -> String {
	match name {
		Some(n) => format!("module-{}", n.to_lowercase().replace(' ', "-")),
		None => format!("module-{}", uuid::Uuid::new_v4()),
	}
}

fn generate_instance_id() -> String { format!("instance-{}", uuid::Uuid::new_v4()) }

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_module_loader_creation() {
		let runtime = Arc::new(WASMRuntime::new(WASMConfig::default()).await.unwrap());
		let config = WASMConfig::default();
		let loader = ModuleLoaderImpl::new(runtime, config);

		// Just test creation
		assert_eq!(loader.get_loaded_modules().await.len(), 0);
	}

	#[test]
	fn test_module_load_options_default() {
		let options = ModuleLoadOptions::default();
		assert_eq!(options.validate, true);
		assert_eq!(options.enable_cache, true);
	}

	#[test]
	fn test_generate_module_id() {
		let id1 = generate_module_id(&Some("Test Module".to_string()));
		let id2 = generate_module_id(&None);

		assert!(id1.starts_with("module-"));
		assert!(id2.starts_with("module-"));
		assert_ne!(id1, id2);
	}
}

// Add uuid dependency to Cargo.toml if needed
// uuid = { version = "1.6", features = ["v4"] }
