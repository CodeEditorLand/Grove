//! HostFunctionRegistry — extracted from FunctionExport module.

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

pub struct HostFunctionRegistry {
	/// Registered host functions
	functions:Arc<RwLock<HashMap<String, RegisteredHostFunction>>>,

	/// Associated host bridge
	bridge:Arc<HostBridge>,
}
