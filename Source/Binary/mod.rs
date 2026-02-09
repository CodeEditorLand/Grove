//! Binary Module
//!
//! Contains binary-specific initialization and build logic.
//! Used by the standalone Grove executable.

pub mod Build;
pub mod Main;

// Re-exports for convenience
pub use Build::{RuntimeBuild, ServiceRegister};
pub use Main::Entry;

/// Binary configuration
#[derive(Debug, Clone)]
pub struct BinaryConfig {
	/// Binary name
	pub name:String,
	/// Binary version
	pub version:String,
	/// Enable verbose output
	pub verbose:bool,
	/// Enable debug mode
	pub debug:bool,
}

impl BinaryConfig {
	/// Create a new binary configuration
	pub fn new() -> Self {
		Self {
			name:"grove".to_string(),
			version:env!("CARGO_PKG_VERSION").to_string(),
			verbose:false,
			debug:cfg!(debug_assertions),
		}
	}

	/// Set verbose mode
	pub fn with_verbose(mut self, verbose:bool) -> Self {
		self.verbose = verbose;
		self
	}

	/// Set debug mode
	pub fn with_debug(mut self, debug:bool) -> Self {
		self.debug = debug;
		self
	}
}

impl Default for BinaryConfig {
	fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_binary_config_default() {
		let config = BinaryConfig::default();
		assert_eq!(config.name, "grove");
		assert!(!config.verbose);
	}

	#[test]
	fn test_binary_config_builder() {
		let config = BinaryConfig::default().with_verbose(true).with_debug(true);

		assert!(config.verbose);
		assert!(config.debug);
	}
}
