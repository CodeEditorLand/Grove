//! Service Register Module
//!
//! Handles service registration with Mountain.
//! Provides gRPC-based service discovery and registration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::dev_log;

use crate::Protocol::{ProtocolConfig, SpineConnection::SpineConnectionImpl};

/// Service register for managing Grove's registration with Mountain
pub struct ServiceRegister;

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
	/// Service name
	pub name:String,
	/// Service type
	pub service_type:ServiceType,
	/// Service version
	pub version:String,
	/// Service endpoint
	pub endpoint:String,
	/// Service capabilities
	pub capabilities:Vec<String>,
	/// Metadata
	pub metadata:serde_json::Value,
}

/// Service type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
	/// Extension host service
	ExtensionHost = 0,
	/// Configuration service
	Configuration = 1,
	/// Logging service
	Logging = 2,
	/// Custom service
	Custom = 99,
}

/// Service registration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistrationResult {
	/// Registration success
	pub success:bool,
	/// Service ID assigned by Mountain
	pub service_id:Option<String>,
	/// Error message if registration failed
	pub error:Option<String>,
	/// Timestamp
	pub timestamp:u64,
}

impl ServiceRegister {
	/// Register Grove with Mountain
	pub async fn register_with_mountain(
		service_name:&str,
		mountain_address:&str,
		auto_reconnect:bool,
	) -> Result<ServiceRegistrationResult> {
		dev_log!("grove", "Registering service '{}' with Mountain at {}", service_name, mountain_address);

		// Create Spine configuration
		let spine_config = ProtocolConfig::new().with_mountain_endpoint(service_name.to_string());

		// Create Spine connection
		let mut connection = SpineConnectionImpl::new(spine_config);

		// Connect to Mountain
		connection.Connect().await.context("Failed to connect to Mountain")?;

		// Prepare registration information
		let registration = ServiceRegistration {
			name:service_name.to_string(),
			service_type:ServiceType::ExtensionHost,
			version:env!("CARGO_PKG_VERSION").to_string(),
			endpoint:mountain_address.to_string(),
			capabilities:vec![
				"wasm-runtime".to_string(),
				"native-rust".to_string(),
				"cocoon-compatible".to_string(),
			],
			metadata:serde_json::json!({
				"host_type": "grove",
				"features": ["wasm", "native", "ipc"]
			}),
		};

		dev_log!("grove", "Service registration: {:?}", registration);

		// Send registration request (placeholder - in real implementation, use gRPC)
		let result = ServiceRegistrationResult {
			success:true,
			service_id:Some(format!("grove-{}", uuid::Uuid::new_v4())),
			error:None,
			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_secs())
				.unwrap_or(0),
		};

		dev_log!("grove", "Service registration result: {:?}", result);

		Ok(result)
	}

	/// Unregister Grove from Mountain
	pub async fn unregister_from_mountain(service_id:&str) -> Result<()> {
		dev_log!("grove", "Unregistering service from Mountain: {}", service_id);

		// Placeholder - in real implementation, call Mountain's unregister service
		dev_log!("grove", "Service unregistered: {}", service_id);

		Ok(())
	}

	/// Heartbeat to keep service alive
	pub async fn send_heartbeat(service_id:&str) -> Result<()> {
		dev_log!("grove", "Sending heartbeat for service: {}", service_id);

		// Placeholder - in real implementation, send heartbeat to Mountain
		Ok(())
	}

	/// Update service information
	pub async fn update_registration(
		service_id:&str,
		registration:ServiceRegistration,
	) -> Result<ServiceRegistrationResult> {
		dev_log!("grove", "Updating service registration: {}", service_id);

		dev_log!("grove", "Updated registration: {:?}", registration);

		Ok(ServiceRegistrationResult {
			success:true,
			service_id:Some(service_id.to_string()),
			error:None,
			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_secs())
				.unwrap_or(0),
		})
	}

	/// Query service information
	pub async fn query_service(service_id:&str) -> Result<ServiceRegistration> {
		dev_log!("grove", "Querying service information: {}", service_id);

		// Placeholder - in real implementation, query Mountain for service info
		Ok(ServiceRegistration {
			name:service_id.to_string(),
			service_type:ServiceType::ExtensionHost,
			version:"0.1.0".to_string(),
			endpoint:"127.0.0.1:50050".to_string(),
			capabilities:Vec::new(),
			metadata:serde_json::Value::Null,
		})
	}

	/// List all registered services
	pub async fn list_services() -> Result<Vec<ServiceRegistration>> {
		dev_log!("grove", "Listing all registered services");

		// Placeholder - in real implementation, query Mountain for all services
		Ok(Vec::new())
	}

	/// Start heartbeat loop
	pub async fn start_heartbeat_loop(service_id:&str, interval_sec:u64) -> Result<()> {
		dev_log!("grove", "Starting heartbeat loop for service: {} (interval: {}s)", service_id, interval_sec);

		let service_id_owned = service_id.to_string();
		tokio::spawn(async move {
			loop {
				tokio::time::sleep(tokio::time::Duration::from_secs(interval_sec)).await;
				if let Err(e) = Self::send_heartbeat(&service_id_owned).await {
					dev_log!("grove", "warn: heartbeat failed: {}", e);
				}
			}
		});

		Ok(())
	}
}

impl Default for ServiceRegister {
	fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_service_register_default() {
		let register = ServiceRegister::default();
		// Just test that it can be created
		let _ = register;
	}

	#[test]
	fn test_service_type() {
		assert_eq!(ServiceType::ExtensionHost as i32, 0);
		assert_eq!(ServiceType::Configuration as i32, 1);
		assert_eq!(ServiceType::Logging as i32, 2);
		assert_eq!(ServiceType::Custom as i32, 99);
	}

	#[tokio::test]
	async fn test_service_registration_creation() {
		let registration = ServiceRegistration {
			name:"test-service".to_string(),
			service_type:ServiceType::ExtensionHost,
			version:"1.0.0".to_string(),
			endpoint:"127.0.0.1:50050".to_string(),
			capabilities:vec!["test-capability".to_string()],
			metadata:serde_json::Value::Null,
		};

		assert_eq!(registration.name, "test-service");
		assert_eq!(registration.service_type, ServiceType::ExtensionHost);
	}
}
