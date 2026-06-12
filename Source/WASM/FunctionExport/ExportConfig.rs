//! ExportConfig — extracted from FunctionExport module.

//! Function Export Module
//!
//! Handles exporting host functions to WASM modules.
//! Provides registration and management of functions that WASM can call.

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use wasmtime::{Caller, Linker};

use crate::{
	WASM::HostBridge::{
		FunctionSignature,
		HostBridgeImpl,
		HostBridgeImpl as HostBridge,
		HostFunctionCallback,
		ParamType,
		ReturnType,
	},
	dev_log,
};

/// Host function registry for WASM exports

pub struct ExportConfig {
	/// Enable function export by default
	pub auto_export:bool,

	/// Enable timing statistics
	pub enable_stats:bool,

	/// Maximum number of functions that can be exported
	pub max_functions:usize,

	/// Function name prefix for exports
	pub name_prefix:Option<String>,
}
