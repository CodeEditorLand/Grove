//! Services Module
//!
//! Provides various services for Grove operation.
//! Includes configuration service, logging service, and more.

pub mod ConfigurationService;

// Re-exports for convenience - use module prefix to avoid E0255 conflicts
// Note: ConfigurationService must be accessed via
// ConfigurationService::ConfigurationServiceImpl

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {

	/// Enable service
	pub enabled:bool,

	/// Service name
	pub name:String,
}

/// Service trait
pub trait Service: Send + Sync {

	/// Get service name
	fn name(&self) -> &str;

	/// Start the service
	fn start(&self) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

	/// Stop the service
	fn stop(&self) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

	/// Check if service is running
	fn is_running(&self) -> impl std::future::Future<Output = bool> + Send;
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_service_config() {
		let config = ServiceConfig { enabled:true, name:"test-service".to_string() };

		assert_eq!(config.name, "test-service");

		assert!(config.enabled);
	}
}
