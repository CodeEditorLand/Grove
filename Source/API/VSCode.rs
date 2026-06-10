//! VS Code API Facade Module
//!
//! Provides the VS Code API facade for Grove extensions.
//! This implements the interface described in vscode.d.ts for extension
//! compatibility.

use std::sync::{
	Arc,
	Mutex,
	atomic::{AtomicU32, Ordering},
};

use serde::{Deserialize, Serialize};

use crate::{API::Types::*, Transport::Strategy::Transport, dev_log};

// ============================================================================
// Provider Registration Store
// ============================================================================

/// Tracks all active language provider registrations with their handles.
///
/// A registration is added when an extension calls `register_*_provider` and
/// removed when `Disposable::dispose()` is called on the returned handle.
#[derive(Debug, Default)]
struct ProviderStore {
	/// Map from handle → (provider_type, selector) for diagnostics.
	entries:Mutex<std::collections::HashMap<u32, (String, String)>>,

	/// Monotonically increasing handle counter.
	next_handle:AtomicU32,
}

impl ProviderStore {
	/// Returns the next unique handle and inserts a registration record.
	fn insert(&self, provider_type:&str, selector:&str) -> u32 {
		let Handle = self.next_handle.fetch_add(1, Ordering::Relaxed);

		if let Ok(mut Guard) = self.entries.lock() {
			Guard.insert(Handle, (provider_type.to_string(), selector.to_string()));
		}

		Handle
	}

	/// Removes a registration by handle (called from Disposable::dispose).
	fn remove(&self, handle:u32) {
		if let Ok(mut Guard) = self.entries.lock() {
			Guard.remove(&handle);
		}
	}

	/// Returns the number of active registrations.
	fn len(&self) -> usize { self.entries.lock().map(|G| G.len()).unwrap_or(0) }
}

/// VS Code API facade - the main entry point for extensions
#[derive(Debug, Clone)]
pub struct VSCodeAPI {
	/// Commands namespace
	pub commands:Arc<CommandNamespace>,

	/// Window namespace
	pub window:Arc<Window>,

	/// Workspace namespace
	pub workspace:Arc<Workspace>,

	/// Languages namespace
	pub languages:Arc<LanguageNamespace>,

	/// Extensions namespace
	pub extensions:Arc<ExtensionNamespace>,

	/// Environment namespace
	pub env:Arc<Env>,
}

impl VSCodeAPI {
	/// Create a new VS Code API facade (no transport - registrations stored
	/// locally only)
	pub fn new() -> Self {
		Self {
			commands:Arc::new(CommandNamespace::new()),

			window:Arc::new(Window::new()),

			workspace:Arc::new(Workspace::new()),

			languages:Arc::new(LanguageNamespace::new()),

			extensions:Arc::new(ExtensionNamespace::new()),

			env:Arc::new(Env::new()),
		}
	}

	/// Create a VS Code API facade wired to a Mountain transport.
	/// Provider registrations will be forwarded to Mountain via
	/// `send_no_response`.
	pub fn new_with_transport(transport:Arc<Transport>) -> Self {
		Self {
			commands:Arc::new(CommandNamespace::new_with_transport(Arc::clone(&transport))),

			window:Arc::new(Window::new_with_transport(Arc::clone(&transport))),

			workspace:Arc::new(Workspace::new_with_transport(Arc::clone(&transport))),

			languages:Arc::new(LanguageNamespace::new_with_transport(Arc::clone(&transport))),

			extensions:Arc::new(ExtensionNamespace::new()),

			env:Arc::new(Env::new()),
		}
	}
}

impl Default for VSCodeAPI {
	fn default() -> Self { Self::new() }
}

/// Commands namespace
#[derive(Debug, Clone)]
pub struct CommandNamespace {
	/// Optional transport to Mountain for command forwarding.
	transport:Option<Arc<Transport>>,
}

impl CommandNamespace {
	/// Create a new CommandNamespace instance
	pub fn new() -> Self { Self }

	/// Register a command
	pub fn register_command(&self, command_id:String, _callback:CommandCallback) -> Result<Command, String> {
		Ok(Command { id:command_id.clone() })
	}

	/// Execute a command by forwarding `commands:execute` to Mountain.
	/// When no transport is wired, returns an error so callers can fall back.
	pub async fn execute_command<T:serde::de::DeserializeOwned>(
		&self,

		command_id:String,

		args:Vec<serde_json::Value>,
	) -> Result<T, String> {
		let Some(transport) = &self.transport else {
			return Err(format!("No transport - cannot execute command: {}", command_id));
		};

		let payload = serde_json::to_vec(&serde_json::json!({
			"method": "commands:execute",
			"parameters": [command_id, args],
		}))
		.map_err(|e| format!("serialise: {}", e))?;

		let response = transport.send(&payload).await.map_err(|e| format!("transport: {}", e))?;

		serde_json::from_slice::<T>(&response).map_err(|e| format!("deserialise: {}", e))
	}
}

/// Command callback type
pub type CommandCallback = Box<dyn Fn(Vec<serde_json::Value>) -> Result<serde_json::Value, String> + Send + Sync>;

/// Command representation
#[derive(Debug, Clone)]
pub struct Command {
	/// The unique identifier of the command
	pub id:String,
}

/// Window namespace
#[derive(Debug, Clone)]
pub struct Window {
	/// Transport for forwarding notifications to Mountain
	transport:Option<Arc<Transport>>,
}

impl Window {
	/// Create a new Window instance
	pub fn new() -> Self { Self { transport:None } }

	/// Create a Window instance with a transport for notification forwarding
	pub fn new_with_transport(transport:Arc<Transport>) -> Self { Self { transport:Some(transport) } }

	/// Show an information message
	pub async fn show_information_message(&self, message:String) -> Result<String, String> {
		if let Some(t) = &self.transport {
			let msg =
				serde_json::json!({"method":"notification:show","parameters":{"message":message,"severity":"info"}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}

		Ok("OK".to_string())
	}

	/// Show a warning message
	pub async fn show_warning_message(&self, message:String) -> Result<String, String> {
		if let Some(t) = &self.transport {
			let msg =
				serde_json::json!({"method":"notification:show","parameters":{"message":message,"severity":"warning"}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}

		Ok("OK".to_string())
	}

	/// Show an error message
	pub async fn show_error_message(&self, message:String) -> Result<String, String> {
		if let Some(t) = &self.transport {
			let msg =
				serde_json::json!({"method":"notification:show","parameters":{"message":message,"severity":"error"}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}

		Ok("OK".to_string())
	}

	/// Create and show a new output channel
	pub fn create_output_channel(&self, name:String) -> OutputChannel {
		match &self.transport {
			Some(t) => OutputChannel::new_with_transport(name, Arc::clone(t)),

			None => OutputChannel::new(name),
		}
	}
}

/// Output channel for logging
#[derive(Debug, Clone)]
pub struct OutputChannel {
	/// The name of the output channel
	name:String,

	/// Transport for forwarding show/hide/dispose notifications to Mountain
	transport:Option<Arc<Transport>>,
}

impl OutputChannel {
	/// Create a new output channel
	///
	/// # Arguments
	///
	/// * `name` - The name of the output channel
	pub fn new(name:String) -> Self { Self { name, transport:None } }

	/// Create a new output channel with a transport for notification forwarding
	pub fn new_with_transport(name:String, transport:Arc<Transport>) -> Self {
		Self { name, transport:Some(transport) }
	}

	/// Append a line to the channel
	pub fn append_line(&self, line:&str) {
		dev_log!("output", "[{}] {}", self.name, line);
	}

	/// Append to the channel
	pub fn append(&self, value:&str) {
		dev_log!("output", "[{}] {}", self.name, value);
	}

	/// Show the output channel
	pub fn show(&self) {
		if let Some(t) = &self.transport {
			let msg = serde_json::json!({"method":"output:show","parameters":{"channel":self.name}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}
	}

	/// Hide the output channel
	pub fn hide(&self) {
		if let Some(t) = &self.transport {
			let msg = serde_json::json!({"method":"output:hide","parameters":{"channel":self.name}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}
	}

	/// Dispose the output channel
	pub fn dispose(&self) {
		if let Some(t) = &self.transport {
			let msg = serde_json::json!({"method":"output:dispose","parameters":{"channel":self.name}});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}
	}
}

/// Workspace namespace
#[derive(Debug, Clone)]
pub struct Workspace {
	/// Optional transport to Mountain for configuration calls.
	transport:Option<Arc<Transport>>,
}

impl Workspace {
	/// Create a new Workspace instance
	pub fn new() -> Self { Self { transport:None } }

	/// Create a Workspace wired to a Mountain transport.
	pub fn new_with_transport(transport:Arc<Transport>) -> Self { Self { transport:Some(transport) } }

	/// Get workspace folders (sync).
	///
	/// Returns an empty vec when called from a synchronous context. Use
	/// `workspace_folders_async` from an async context to retrieve live data
	/// from Mountain.
	pub fn workspace_folders(&self) -> Vec<WorkspaceFolder> {
		dev_log!(
			"workspace",
			"[Workspace::workspace_folders] transport wired but called synchronously"
		);

		Vec::new()
	}

	/// Get workspace folders by querying Mountain via the transport.
	///
	/// Falls back to an empty vec on any transport or parse error.
	pub async fn workspace_folders_async(&self) -> Vec<WorkspaceFolder> {
		let Some(t) = &self.transport else {
			return Vec::new();
		};

		let msg = serde_json::json!({"method":"workspaces:getFolders","parameters":{}});

		let Ok(bytes) = serde_json::to_vec(&msg) else {
			return Vec::new();
		};

		match t.send(&bytes).await {
			Ok(response) => serde_json::from_slice::<Vec<WorkspaceFolder>>(&response).unwrap_or_default(),

			Err(e) => {
				dev_log!("workspace", "[workspace_folders_async] error: {}", e);

				Vec::new()
			},
		}
	}

	/// Get workspace configuration
	pub fn get_configuration(&self, section:Option<String>) -> WorkspaceConfiguration {
		match &self.transport {
			Some(t) => WorkspaceConfiguration::new_with_transport(section, Arc::clone(t)),

			None => WorkspaceConfiguration::new(section),
		}
	}
}

/// Workspace folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
	/// The uri of the workspace folder
	pub uri:String,

	/// The name of the workspace folder
	pub name:String,

	/// The ordinal number of the workspace folder
	pub index:u32,
}

/// Workspace configuration
#[derive(Debug, Clone)]
pub struct WorkspaceConfiguration {
	/// The configuration section name
	section:Option<String>,

	/// Optional transport to Mountain for live configuration calls.
	transport:Option<Arc<Transport>>,
}

impl WorkspaceConfiguration {
	/// Create a new workspace configuration
	///
	/// # Arguments
	///
	/// * `section` - Optional section name to retrieve
	pub fn new(section:Option<String>) -> Self { Self { section, transport:None } }

	/// Create a workspace configuration wired to a Mountain transport.
	///
	/// # Arguments
	///
	/// * `section` - Optional section name to retrieve
	/// * `transport` - Active Mountain transport
	pub fn new_with_transport(section:Option<String>, transport:Arc<Transport>) -> Self {
		Self { section, transport:Some(transport) }
	}

	/// Get a configuration value (sync stub - returns Err; use
	/// `get_async` when a transport is available).
	pub fn get<T:serde::de::DeserializeOwned>(&self, _key:String) -> Result<T, String> {
		Err("Configuration not implemented".to_string())
	}

	/// Get a configuration value via Mountain `configuration:get`.
	pub async fn get_async<T:serde::de::DeserializeOwned>(&self, key:String) -> Result<T, String> {
		let Some(t) = &self.transport else {
			return Err("No transport wired".to_string());
		};

		let msg = serde_json::json!({
			"method": "configuration:get",
			"parameters": { "section": self.section, "key": key }
		});

		let bytes = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;

		let response = t.send(&bytes).await.map_err(|e| e.to_string())?;

		serde_json::from_slice::<T>(&response).map_err(|e| e.to_string())
	}

	/// Check if a key exists in the configuration
	pub fn has(&self, _key:String) -> bool { false }

	/// Update a configuration value via Mountain `configuration:update`
	/// (fire-and-forget).
	pub async fn update(&self, key:String, value:serde_json::Value) -> Result<(), String> {
		let Some(t) = &self.transport else {
			return Err("No transport wired".to_string());
		};

		let msg = serde_json::json!({
			"method": "configuration:update",
			"parameters": { "section": self.section, "key": key, "value": value }
		});

		let bytes = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;

		let t2 = Arc::clone(t);

		tokio::spawn(async move {
			let _ = t2.send_no_response(&bytes).await;
		});

		Ok(())
	}
}

/// Languages namespace - mirrors the full vscode.languages API surface.
///
/// Each `register_*_provider` method:
/// 1. Assigns a unique handle from the atomic counter
/// 2. Stores the registration in `ProviderStore` for lifecycle tracking
/// 3. If a Mountain `Transport` is wired, forwards a `send_no_response` JSON
///    notification matching Mountain's `GenericNotification` format so that
///    Mountain can store the provider in its `ProviderRegistry`
/// 4. Returns a `Disposable` that removes the registration on dispose
#[derive(Debug)]
pub struct LanguageNamespace {
	/// Active provider registration store.
	store:Arc<ProviderStore>,

	/// Optional transport to Mountain for forwarding registrations.
	transport:Option<Arc<Transport>>,
}

impl Clone for LanguageNamespace {
	fn clone(&self) -> Self { Self { store:Arc::clone(&self.store), transport:self.transport.clone() } }
}

impl LanguageNamespace {
	/// Create a new LanguageNamespace instance (local storage only).
	pub fn new() -> Self { Self { store:Arc::new(ProviderStore::default()), transport:None } }

	/// Create a new LanguageNamespace wired to a Mountain transport.
	/// Registrations are forwarded via `send_no_response` as JSON
	/// notifications.
	pub fn new_with_transport(transport:Arc<Transport>) -> Self {
		Self { store:Arc::new(ProviderStore::default()), transport:Some(transport) }
	}

	/// Returns the number of active provider registrations.
	pub fn active_registration_count(&self) -> usize { self.store.len() }

	/// Internal helper: register a provider, return a disposable handle.
	fn register(&self, provider_type:&str, selector:&DocumentSelector) -> Disposable {
		let ProviderTypeOwned = provider_type.to_string();

		let SelectorStr = selector
			.iter()
			.filter_map(|F| F.language.as_deref())
			.collect::<Vec<_>>()
			.join(",");

		let Handle = self.store.insert(&ProviderTypeOwned, &SelectorStr);

		let Store = Arc::clone(&self.store);

		dev_log!(
			"extensions",
			"[LanguageNamespace] registered {} handle={} selector={}",
			ProviderTypeOwned,
			Handle,
			SelectorStr
		);

		// Forward registration to Mountain if transport is wired
		if let Some(Transport) = &self.transport {
			let Notification = serde_json::json!({
				"method": format!("register_{}", ProviderTypeOwned),
				"parameters": {
					"handle": Handle,
					"language_selector": SelectorStr,
					"extension_id": "grove-extension",
				}
			});

			if let Ok(Bytes) = serde_json::to_vec(&Notification) {
				let TransportClone = Arc::clone(Transport);

				tokio::spawn(async move {
					let _ = TransportClone.send_no_response(&Bytes).await;
				});
			}
		}

		Disposable::with_callback(Box::new(move || {
			Store.remove(Handle);

			dev_log!(
				"extensions",
				"[LanguageNamespace] disposed {} handle={}",
				ProviderTypeOwned,
				Handle
			);
		}))
	}

	/// Register completion item provider
	pub async fn register_completion_item_provider<T:CompletionItemProvider>(
		&self,

		selector:DocumentSelector,

		_provider:T,

		_trigger_characters:Option<Vec<String>>,
	) -> Result<Disposable, String> {
		Ok(self.register("completion", &selector))
	}

	/// Register hover provider
	pub fn register_hover_provider(&self, selector:DocumentSelector) -> Disposable { self.register("hover", &selector) }

	/// Register definition provider
	pub fn register_definition_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("definition", &selector)
	}

	/// Register reference provider
	pub fn register_reference_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("references", &selector)
	}

	/// Register code actions provider
	pub fn register_code_actions_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("codeAction", &selector)
	}

	/// Register document highlight provider
	pub fn register_document_highlight_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("documentHighlight", &selector)
	}

	/// Register document symbol provider
	pub fn register_document_symbol_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("documentSymbol", &selector)
	}

	/// Register workspace symbol provider
	pub fn register_workspace_symbol_provider(&self) -> Disposable { self.register("workspaceSymbol", &Vec::new()) }

	/// Register rename provider
	pub fn register_rename_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("rename", &selector)
	}

	/// Register document formatting provider
	pub fn register_document_formatting_edit_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("documentFormatting", &selector)
	}

	/// Register document range formatting provider
	pub fn register_document_range_formatting_edit_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("documentRangeFormatting", &selector)
	}

	/// Register on-type formatting provider
	pub fn register_on_type_formatting_edit_provider(
		&self,

		selector:DocumentSelector,

		_trigger_characters:Vec<String>,
	) -> Disposable {
		self.register("onTypeFormatting", &selector)
	}

	/// Register signature help provider
	pub fn register_signature_help_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("signatureHelp", &selector)
	}

	/// Register code lens provider
	pub fn register_code_lens_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("codeLens", &selector)
	}

	/// Register folding range provider
	pub fn register_folding_range_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("foldingRange", &selector)
	}

	/// Register selection range provider
	pub fn register_selection_range_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("selectionRange", &selector)
	}

	/// Register semantic tokens provider
	pub fn register_document_semantic_tokens_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("semanticTokens", &selector)
	}

	/// Register inlay hints provider
	pub fn register_inlay_hints_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("inlayHints", &selector)
	}

	/// Register type hierarchy provider
	pub fn register_type_hierarchy_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("typeHierarchy", &selector)
	}

	/// Register call hierarchy provider
	pub fn register_call_hierarchy_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("callHierarchy", &selector)
	}

	/// Register linked editing range provider
	pub fn register_linked_editing_range_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("linkedEditingRange", &selector)
	}

	/// Register declaration provider
	pub fn register_declaration_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("declaration", &selector)
	}

	/// Register implementation provider
	pub fn register_implementation_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("implementation", &selector)
	}

	/// Register type definition provider
	pub fn register_type_definition_provider(&self, selector:DocumentSelector) -> Disposable {
		self.register("typeDefinition", &selector)
	}

	/// Register diagnostic collection
	pub fn create_diagnostic_collection(&self, name:Option<String>) -> DiagnosticCollection {
		match &self.transport {
			Some(t) => DiagnosticCollection::new_with_transport(name, Arc::clone(t)),

			None => DiagnosticCollection::new(name),
		}
	}

	/// Set language configuration
	pub fn set_language_configuration(&self, language:String) -> Disposable {
		self.register(
			"languageConfiguration",
			&vec![DocumentFilter { language:Some(language), scheme:None, pattern:None }],
		)
	}
}

/// Document selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFilter {
	/// A language id, like `typescript`
	pub language:Option<String>,

	/// A Uri scheme, like `file` or `untitled`
	pub scheme:Option<String>,

	/// A glob pattern, like `*.{ts,js}`
	pub pattern:Option<String>,
}

/// Document selector type
pub type DocumentSelector = Vec<DocumentFilter>;

/// Completion item provider
pub trait CompletionItemProvider: Send + Sync {
	/// Provide completion items at the given position
	///
	/// # Arguments
	///
	/// * `document` - The text document identifier
	/// * `position` - The position in the document
	/// * `context` - The completion context
	/// * `token` - Optional cancellation token
	///
	/// # Returns
	///
	/// A vector of completion items
	fn provide_completion_items(
		&self,

		document:TextDocumentIdentifier,

		position:Position,

		context:CompletionContext,

		token:Option<String>,
	) -> Vec<CompletionItem>;
}

/// Completion context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionContext {
	/// How the completion was triggered
	#[serde(rename = "triggerKind")]
	pub trigger_kind:CompletionTriggerKind,

	/// The character that triggered the completion
	#[serde(rename = "triggerCharacter")]
	pub trigger_character:Option<String>,
}

/// Completion trigger kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompletionTriggerKind {
	/// Completion was triggered by typing an identifier
	#[serde(rename = "Invoke")]
	Invoke = 0,

	/// Completion was triggered by a trigger character
	#[serde(rename = "TriggerCharacter")]
	TriggerCharacter = 1,

	/// Completion was re-triggered
	#[serde(rename = "TriggerForIncompleteCompletions")]
	TriggerForIncompleteCompletions = 2,
}

/// Diagnostic collection
#[derive(Debug, Clone)]
pub struct DiagnosticCollection {
	/// The name of the diagnostic collection
	name:Option<String>,

	/// Optional transport to Mountain for forwarding diagnostic notifications.
	transport:Option<Arc<Transport>>,
}

impl DiagnosticCollection {
	/// Create a new diagnostic collection
	///
	/// # Arguments
	///
	/// * `name` - Optional name for the collection
	pub fn new(name:Option<String>) -> Self { Self { name, transport:None } }

	/// Create a new diagnostic collection wired to a Mountain transport.
	/// set/delete/clear/dispose calls are forwarded via `send_no_response`.
	pub fn new_with_transport(name:Option<String>, transport:Arc<Transport>) -> Self {
		Self { name, transport:Some(transport) }
	}

	/// Forward a notification to Mountain if a transport is wired.
	fn fire(&self, method:&str, params:serde_json::Value) {
		if let Some(t) = &self.transport {
			let msg = serde_json::json!({"method": method, "parameters": params});

			if let Ok(bytes) = serde_json::to_vec(&msg) {
				let t = Arc::clone(t);

				tokio::spawn(async move {
					let _ = t.send_no_response(&bytes).await;
				});
			}
		}
	}

	/// Set diagnostics for a resource
	pub fn set(&self, uri:String, diagnostics:Vec<Diagnostic>) {
		self.fire("diagnostics:set", serde_json::json!({"uri": uri, "diagnostics": diagnostics}));
	}

	/// Delete diagnostics for a resource
	pub fn delete(&self, uri:String) { self.fire("diagnostics:delete", serde_json::json!({"uri": uri})); }

	/// Clear all diagnostics in this collection
	pub fn clear(&self) { self.fire("diagnostics:clear", serde_json::json!({"name": self.name})); }

	/// Dispose the collection and release Mountain-side resources
	pub fn dispose(&self) { self.fire("diagnostics:dispose", serde_json::json!({"name": self.name})); }
}

/// Disposable resource handle.
///
/// Returned by all `register_*_provider` methods. Calling `dispose()` removes
/// the provider registration from the `LanguageNamespace` store.
pub struct Disposable {
	callback:Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl std::fmt::Debug for Disposable {
	fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Disposable")
			.field("has_callback", &self.callback.is_some())
			.finish()
	}
}

impl Clone for Disposable {
	/// Cloning a Disposable produces a no-op copy.
	/// The original disposable retains the callback.
	fn clone(&self) -> Self { Self { callback:None } }
}

impl Disposable {
	/// Create a no-op disposable.
	pub fn new() -> Self { Self { callback:None } }

	/// Create a disposable with a callback invoked on `dispose()`.
	pub fn with_callback(callback:Box<dyn FnOnce() + Send + Sync>) -> Self { Self { callback:Some(callback) } }

	/// Dispose the resource, invoking the registered callback if present.
	pub fn dispose(mut self) {
		if let Some(Callback) = self.callback.take() {
			Callback();
		}
	}
}

impl Default for Disposable {
	fn default() -> Self { Self::new() }
}

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

/// Environment namespace
#[derive(Debug, Clone)]
pub struct Env;

impl Env {
	/// Create a new Env instance
	pub fn new() -> Self { Self }

	/// Get environment variable
	pub fn get_env_var(&self, name:String) -> Option<String> { std::env::var(name).ok() }

	/// Check if running on a specific platform
	pub fn is_windows(&self) -> bool { cfg!(windows) }

	/// Check if running on macOS
	pub fn is_mac(&self) -> bool { cfg!(target_os = "macos") }

	/// Check if running on Linux
	pub fn is_linux(&self) -> bool { cfg!(target_os = "linux") }

	/// Get the app name
	pub fn app_name(&self) -> String { "VS Code".to_string() }

	/// Get the app root
	pub fn app_root(&self) -> Option<String> { std::env::var("VSCODE_APP_ROOT").ok() }
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_vscode_api_creation() {
		let _api = VSCodeAPI::new();

		// Arc fields are always initialized, so just verify creation works
	}

	#[test]
	fn test_position_operations() {
		let pos = Position::new(5, 10);

		assert_eq!(pos.line, 5);

		assert_eq!(pos.character, 10);
	}

	#[test]
	fn test_output_channel() {
		let channel = OutputChannel::new("test".to_string());

		channel.append_line("test message");
	}

	#[test]
	fn test_disposable() {
		let disposable = Disposable::new();

		disposable.dispose();
	}
}
