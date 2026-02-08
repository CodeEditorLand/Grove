//! Grove - Rust/WASM Extension Host for VS Code
//!
//! Grove provides a secure, sandboxed environment for running VS Code extensions
//! compiled to WebAssembly or native Rust. It complements Cocoon (Node.js) by
//! offering a native extension host with full WASM support.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │          Extension Host                 │
//! ├─────────────────────────────────────────┤
//! │  Extension Manager  →  Activation Engine │
//! │  API Bridge         →  VS Code API      │
//! └───────────────────┬─────────────────────┘
//!                     │
//! ┌───────────────────▼─────────────────────┐
//! │          WASM Runtime (Wasmtime)        │
//! │  Module Loader  →  Host Bridge         │
//! └───────────────────┬─────────────────────┘
//!                     │
//! ┌───────────────────▼─────────────────────┐
//! │        Transport Layer                  │
//! │  gRPC  |  IPC  |  Direct WASM          │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Features
//!
//! - **Standalone Operation**: Run independently or connect to Mountain via gRPC
//! - **WASM Support**: Full WebAssembly runtime with Wasmtime
//! - **Multiple Transport**: gRPC, IPC, and direct WASM communication
//! - **Secure Sandboxing**: Wasmtime-based isolation for untrusted extensions
//! - **Cocoon Compatible**: Shares API surface with Node.js host
//!
//! # Example: Standalone Usage
//!
//! ```rust,no_run
//! use grove::{ExtensionHost, Transport};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let host = ExtensionHost::new(Transport::default()).await?;
//!     host.load_extension("/path/to/extension").await?;
//!     host.activate().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Module Organization
//!
//! - [`Host`] - Extension hosting core (ExtensionHost, ExtensionManager, etc.)
//! - [`WASM`] - WebAssembly runtime integration
//! - [`Transport`] - Communication strategies (gRPC, IPC, WASM)
//! - [`API`] - VS Code API facade and types
//! - [`Protocol`] - Protocol handling (Spine connection)
//! - [`Services`] - Host services (configuration, etc.)
//! - [`Common`] - Shared utilities and error types

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]

// Public module declarations
pub mod API;
pub mod Binary;
pub mod Common;
pub mod Host;
pub mod Protocol;
pub mod Services;
pub mod Transport;
pub mod WASM;

// Re-exports for convenience
pub use API::vscode;
pub use API::types;
pub use Common::error::{GroveError, GroveResult};
pub use Common::traits::ExtensionContext;
pub use Host::ExtensionHost;
pub use Host::ExtensionManager;
pub use Transport::Strategy;
pub use Transport::Transport;
pub use WASM::Runtime;

// Library version
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Grove library information
#[derive(Debug, Clone)]
pub struct GroveInfo {
    /// Version string
    pub version: &'static str,
    /// Build timestamp
    build_timestamp: String,
}

impl GroveInfo {
    /// Create new GroveInfo with current build information
    pub fn new() -> Self {
        Self {
            version: VERSION,
            build_timestamp: env!("VERGEN_BUILD_TIMESTAMP").to_string(),
        }
    }

    /// Get the Grove version
    pub fn version(&self) -> &'static str {
        self.version
    }
}

impl Default for GroveInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize Grove library
///
/// This sets up logging and other global state.
/// Call once at application startup.
pub fn init() -> anyhow::Result<()> {
    // Initialize tracing subscriber with environment-based filtering
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;

    tracing::info!("Grove v{} initialized", VERSION);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.'));
    }

    #[test]
    fn test_grove_info() {
        let info = GroveInfo::new();
        assert_eq!(info.version(), VERSION);
    }
}
