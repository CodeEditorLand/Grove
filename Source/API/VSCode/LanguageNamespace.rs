use std::sync::Arc;

use serde_json;

use crate::{Transport::Strategy::Transport, dev_log};
use super::{
	DiagnosticCollection::DiagnosticCollection,
	Disposable::Disposable,
	DocumentSelector::DocumentSelector,
	ProviderStore::ProviderStore,
};

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
