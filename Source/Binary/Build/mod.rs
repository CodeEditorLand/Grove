//! Build Module (Binary)
//!
//! Provides runtime construction and service registration.
//! Used by the standalone Grove executable.

pub mod RuntimeBuild;
pub mod ServiceRegister;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exists() {
        // Test that module can be imported
        let _ = RuntimeBuild;
        let _ = ServiceRegister;
    }
}
