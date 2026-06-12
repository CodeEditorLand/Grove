//! Connection status
//!
//!  ☀️ 🟡 MOUNTAIN_GROVE_WASM

use crate::Protocol::SpineActionClient::HostInfo;

/// Connection status
///
///  ☀️ 🟡 MOUNTAIN_GROVE_WASM
#[derive(Clone, Debug)]
pub struct ConnectionStatus {
	pub connected:bool,

	pub host_id:String,

	pub uptime:Option<i64>,

	pub host_info:Option<HostInfo>,
}
