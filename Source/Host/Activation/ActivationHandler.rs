//! Activation handler for an extension

use crate::Host::Activation::ActivationEvent;

/// Activation handler for an extension
#[derive(Debug, Clone)]
pub(crate) struct ActivationHandler {
	/// Extension ID
	pub extension_id:String,

	/// Activation events
	pub events:Vec<ActivationEvent::ActivationEvent>,

	/// Activation function path
	pub activation_function:String,

	/// Whether extension is currently active
	pub is_active:bool,

	/// Last activation time
	pub last_activation:Option<u64>,
}
