//! Grove host capabilities
//!
//!  ☀️ 🟡 MOUNTAIN_GROVE_WASM

/// Grove host capabilities
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
#[derive(Clone, Debug)]
pub struct GroveCapabilities {
	/// Supports WASM
	pub wasm_enabled:bool,

	/// Supports Rhai
	pub rhai_enabled:bool,

	/// Supports Rhai native bridge
	pub native_bridge_enabled:bool,

	/// Maximum WASM memory (MB)
	pub wasm_memory_limit_mb:u32,

	/// Maximum concurrent Rhai scripts
	pub max_rhai_scripts:u32,

	/// Supported extension packages
	pub supported_extensions:Vec<String>, // ['wsix', 'rix', 'lsix']
}
