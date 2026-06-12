#![allow(
	non_snake_case,
	non_camel_case_types,
	non_upper_case_globals,
	dead_code,
	unused_imports,
	unused_variables,
	unused_assignments,
	unexpected_cfgs
)]
#![warn(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]

//! # Grove — Rust/WASM Extension Host for VS Code
//!
//! Grove provides a secure, sandboxed environment for running VS Code
//! extensions compiled to WebAssembly or native Rust. It complements the
//! Node.js-based extension host (Cocoon) by offering a native extension
//! host with full WASM support via WASMtime.
//!
//! # Architecture
//!
//! ```text
//! +++++++++++++++++++++++++++++++++++++++++++
//! +          Extension Host                 +
//! +++++++++++++++++++++++++++++++++++++++++++
//! +  Extension Manager  →  Activation Engine +
//! +  API Bridge         →  VS Code API      +
//! +++++++++++++++++++++++++++++++++++++++++++
//!                     +
//! ++++++++++++++++++++▼++++++++++++++++++++++
//! +          WASM Runtime (WASMtime)        +
//! +  Module Loader  →  Host Bridge         +
//! +++++++++++++++++++++++++++++++++++++++++++
//!                     +
//! ++++++++++++++++++++▼++++++++++++++++++++++
//! +        Transport Layer                  +
//! +  gRPC  |  IPC  |  Direct WASM          +
//! +++++++++++++++++++++++++++++++++++++++++++
//! ```
//!
//! # Features
//!
//! - **Standalone Operation** — Run independently or connect to Mountain via
//!   gRPC
//! - **WASM Support** — Full WebAssembly runtime with WASMtime
//! - **Multiple Transports** — gRPC, IPC, and direct WASM communication
//! - **Secure Sandboxing** — WASMtime-based isolation for untrusted extensions
//! - **Cocoon-Compatible** — Shares API surface with the Node.js extension host
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use grove::{ExtensionHost, Transport};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//! 	let host = ExtensionHost::new(Transport::default()).await?;
//! 	host.load_extension("/path/to/extension").await?;
//! 	host.activate().await?;
//! 	Ok(())
//! }
//! ```
//!
//! # Module Organization
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`Host`] | Extension hosting core (ExtensionHost, ExtensionManager, Activation) |
//! | [`WASM`] | WebAssembly runtime integration (WASMtime) |
//! | [`Transport`] | Communication strategies (gRPC, IPC, WASM) |
//! | [`API`] | VS Code API facade and common types |
//! | [`Protocol`] | Spine protocol and Mountain communication |
//! | [`Services`] | Host services (configuration, etc.) |
//! | [`Common`] | Shared utilities, traits, and error types |
//! | [`Binary`] | Standalone executable entry points |
//! | [`DevLog`] | Tag-filtered development logging |

// Public module declarations
pub mod Binary;

pub mod DevLog;

pub mod Host;

pub mod Protocol;

pub mod Transport;

pub mod WASM;

/// Library version string (from `CARGO_PKG_VERSION`).
const VERSION:&str = env!("CARGO_PKG_VERSION");

/// Grove library metadata.
#[derive(Debug, Clone)]
pub struct Struct {
	/// Library version string.
	pub version:&'static str,

	/// Build timestamp (embedded at compile time via `VERGEN_BUILD_TIMESTAMP`).
	#[allow(dead_code)]
	build_timestamp:String,
}

impl Struct {
	/// Creates a new `GroveInfo` with the current build information.
	pub fn new() -> Self { Self { version:VERSION, build_timestamp:env!("VERGEN_BUILD_TIMESTAMP").to_string() } }

	/// Returns the Grove library version string.
	pub fn version(&self) -> &'static str { self.version }
}

impl Default for Struct {
	fn default() -> Self { Self::new() }
}

/// Initializes the Grove library.
///
/// Sets up logging and other global state. Should be called once at
/// application startup.
///
/// ## Errors
///
/// Returns an error if initialization fails (e.g., logger setup failure).
pub fn init() -> anyhow::Result<()> {
	use crate::dev_log;

	dev_log!("grove", "Grove v{} initialized", VERSION);

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
