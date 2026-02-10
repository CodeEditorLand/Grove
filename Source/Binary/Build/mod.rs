//! Build Module (Binary)
//!
//! Provides runtime construction and service registration.
//! Used by the standalone Grove executable.

mod RuntimeBuildMod;
mod ServiceRegisterMod;

// Re-export public structs from submodules
pub use RuntimeBuildMod::RuntimeBuild;
pub use ServiceRegisterMod::ServiceRegister;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_module_exists() {
		// Test that module can be imported
		let _ = RuntimeBuild;
		let _ = ServiceRegister;
	}
}
