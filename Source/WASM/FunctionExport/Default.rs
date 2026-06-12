//! Default — extracted from FunctionExport module.

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

impl Default for ExportConfig {
	fn default() -> Self {
		Self {
			auto_export:true,

			enable_stats:true,

			max_functions:1000,

			name_prefix:Some("host_".to_string()),
		}
	}
}
