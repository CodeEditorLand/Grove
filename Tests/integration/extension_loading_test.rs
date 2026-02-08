//! Integration Test: Extension Loading
//!
//! Tests for loading and managing extensions in Grove.

use grove::{ExtensionHost, ExtensionManager, HostConfig, Transport};
use std::path::PathBuf;
use tokio;

/// Test extension loading functionality
#[tokio::test]
async fn test_extension_loading() {
    // Create a minimal extension host
    let transport = Transport::default();
    let host = ExtensionHost::with_config(transport, HostConfig::default())
        .await
        .unwrap();

    // Test listing extensions (should be empty initially)
    let extensions = host.extension_manager().list_extensions().await;
    assert_eq!(extensions.len(), 0);

    // Test that host is ready
    let state = host.state().await;
    // Initially should be Created, then Ready after load
}

/// Test extension activation
#[tokio::test]
async fn test_extension_activation() {
    // Create host
    let transport = Transport::default();
    let host = ExtensionHost::with_config(transport, HostConfig::default())
        .await
        .unwrap();

    // Test activation of non-existent extension should fail
    let result = host.activate("nonexistent.ext").await;
    assert!(result.is_err());
}

/// Test extension manager operations
#[tokio::test]
async fn test_extension_manager_operations() {
    let wasm_runtime = std::sync::Arc::new(
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(grove::WASM::Runtime::WasmRuntime::new(
                grove::WASM::Runtime::WasmConfig::default(),
            ))
            .unwrap()
    );
    let manager = grove::ExtensionManager::new(wasm_runtime, HostConfig::default());

    // Test listing extensions
    let extensions = manager.list_extensions().await;
    assert_eq!(extensions.len(), 0);

    // Test getting non-existent extension
    let ext = manager.get_extension("nonexistent").await;
    assert!(ext.is_none());
}

/// Test WASM runtime creation
#[tokio::test]
async fn test_wasm_runtime_creation() {
    let runtime = grove::WASM::Runtime::WasmRuntime::new(
        grove::WASM::Runtime::WasmConfig::default(),
    )
    .await
    .unwrap();

    // Test that runtime is created
    assert_eq!(runtime.instance_count().await, 0);
}

/// Test transport creation
#[tokio::test]
async fn test_transport_creation() {
    let transport = Transport::default();

    // Test transport type
    let transport_type = transport.transport_type();
    // Should be one of the valid types
    assert!(
        transport_type == grove::Transport::TransportType::Grpc
            || transport_type == grove::Transport::TransportType::Ipc
            || transport_type == grove::Transport::TransportType::Wasm
    );
}

/// Test API types
#[test]
fn test_api_types() {
    use grove::API::types::*;

    let position = Position::new(0, 0);
    assert_eq!(position.line, 0);
    assert_eq!(position.character, 0);

    let range = Range::new(position, position);
    assert_eq!(range.start, position);
    assert_eq!(range.end, position);

    let text_edit = TextEdit::new(range, "test".to_string());
    assert_eq!(text_edit.new_text, "test");
}

/// Test VS Code API facade
#[test]
fn test_vscode_api_facade() {
    let api = grove::vscode!();
    
    assert!(api.commands.is_some());
    assert!(api.window.is_some());
    assert!(api.workspace.is_some());
}

/// Test error handling
#[test]
fn test_error_handling() {
    use grove::GroveError;

    let error = GroveError::extension_not_found("test.ext");
    assert_eq!(error.error_code(), "EXT_NOT_FOUND");

    let error = GroveError::timeout("test_operation", 5000);
    assert!(error.is_transient());
}

/// Test configuration service
#[tokio::test]
async fn test_configuration_service() {
    let service = grove::Services::ConfigurationService::new(None);
    service.start().await.unwrap();

    // Test setting and getting configuration
    service
        .set(
            "test.key".to_string(),
            serde_json::json!("test-value"),
            grove::Services::ConfigurationScope::Global,
        )
        .await
        .unwrap();

    let value = service.get("test.key").await;
    assert_eq!(value, Some(serde_json::json!("test-value")));

    service.stop().await.unwrap();
}

/// Test Spine connection
#[tokio::test]
async fn test_spine_connection() {
    let config = grove::Protocol::SpineConfig::new("test-host".to_string());
    let connection = grove::Protocol::SpineConnection::new(config);

    // Test state (starts as Disconnected)
    let state = connection.get_state().await;
    assert_eq!(
        state,
        grove::Protocol::ConnectionState::Disconnected
    );

    // Test is_connected flag
    let connected = connection.is_connected().await;
    assert!(!connected);
}

/// Build integration test runner
fn main() {
    println!("Running Grove integration tests...");
    
    // Note: These tests are meant to be run with `cargo test --test integration`
    println!("Run integration tests with: cargo test --test integration");
}
