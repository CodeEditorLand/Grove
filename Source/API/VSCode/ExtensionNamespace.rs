use serde::{Deserialize, Serialize};
use serde_json;

/// Extensions namespace
#[derive(Debug, Clone)]
pub struct ExtensionNamespace;

impl ExtensionNamespace {
	/// Create a new ExtensionNamespace instance
	pub fn new() -> Self { Self }

	/// Get all extensions
	pub fn all(&self) -> Vec<Extension> { Vec::new() }

	/// Get an extension by id
	pub fn get_extension(&self, _extension_id:String) -> Option<Extension> { None }
}

/// Extension representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
	/// The canonical extension identifier in the form of `publisher.name`
	pub id:String,

	/// The absolute file path of the directory containing the extension
	#[serde(rename = "extensionPath")]
	pub extension_path:String,

	/// `true` if the extension is enabled
	pub is_active:bool,

	/// The package.json object of the extension
	#[serde(rename = "packageJSON")]
	pub package_json:serde_json::Value,
}
