//! Host information from Mountain
//!
//!  ☀️ 🟡 MOUNTAIN_GROVE_WASM

/// Host information from Mountain
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
#[derive(Clone, Debug)]
pub struct HostInfo {
	pub host_id:String,

	pub host_registry_id:String,

	pub heartbeat_interval_sec:u32,
}
