use serde::{Deserialize, Serialize};

use crate::API::Types::{CompletionItem, Position, TextDocumentIdentifier};

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
