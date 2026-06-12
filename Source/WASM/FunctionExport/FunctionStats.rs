//! FunctionStats — extracted from FunctionExport module.

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

pub struct FunctionStats {
	/// Number of times called
	pub call_count:u64,

	/// Total execution time in nanoseconds
	pub total_execution_ns:u64,

	/// Last call timestamp
	pub last_call_at:Option<u64>,

	/// Number of errors
	pub error_count:u64,
}
