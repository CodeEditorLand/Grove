//! Common API Types Module
//!
//! Defines common types used throughout the VS Code API facade.

use serde::{Deserialize, Serialize};

/// Position in a text document
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Position {
	/// Line position in a document (0-based)
	pub line:u32,

	/// Character offset on a line in a document (0-based)
	pub character:u32,
}

impl Position {
	/// Create a new position
	pub fn new(line:u32, character:u32) -> Self { Self { line, character } }

	/// Position at line 0, character 0
	pub fn zero() -> Self { Self { line:0, character:0 } }

	/// Create a position from a line and column (1-based to 0-based)
	pub fn from_line_column(line:u32, column:u32) -> Self {
		Self { line:line.saturating_sub(1), character:column.saturating_sub(1) }
	}
}

impl Default for Position {
	fn default() -> Self { Self::zero() }
}

/// A range in a text document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
	/// The range's start position
	pub start:Position,

	/// The range's end position
	pub end:Position,
}

impl Range {
	/// Create a new range
	pub fn new(start:Position, end:Position) -> Self { Self { start, end } }

	/// Create a range covering a single position
	pub fn empty(position:Position) -> Self { Self { start:position, end:position } }

	/// Check if the position is in this range
	pub fn contains(&self, position:Position) -> bool { position >= self.start && position <= self.end }
}

impl Default for Range {
	fn default() -> Self { Self::empty(Position::zero()) }
}

/// Represents a location inside a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Location {
	/// The resource identifier of this location
	pub uri:String,

	/// The document range
	pub range:Range,
}

impl Location {
	/// Create a new location
	pub fn new(uri:String, range:Range) -> Self { Self { uri, range } }
}

/// Represents a diagnostic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
	/// The range at which the message applies
	pub range:Range,

	/// The diagnostic's severity
	pub severity:Option<DiagnosticSeverity>,

	/// The diagnostic's code
	pub code:Option<DiagnosticCode>,

	/// A human-readable string describing the source of this diagnostic
	pub source:Option<String>,

	/// The diagnostic's message
	pub message:String,

	/// Additional metadata about the diagnostic
	pub tags:Option<Vec<DiagnosticTag>>,

	/// Related diagnostic information
	pub related_information:Option<Vec<DiagnosticRelatedInformation>>,
}

impl Diagnostic {
	/// Create a new diagnostic
	pub fn new(range:Range, message:String) -> Self {
		Self {
			range,
			severity:None,
			code:None,
			source:None,
			message,
			tags:None,
			related_information:None,
		}
	}
}

/// The severity of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
	/// Not an error, but something to be aware of
	Hint = 3,

	/// Informational
	Information = 2,

	/// Something to warn about
	Warning = 1,

	/// An error
	Error = 0,
}

/// Diagnostic code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DiagnosticCode {
	/// Numeric code
	Number(i64),

	/// String code
	String(String),
}

/// Diagnostic tags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticTag {
	/// Unused or unnecessary code
	Unnecessary = 1,

	/// Deprecated code
	Deprecated = 2,
}

/// Related diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
	/// The location of this related diagnostic information
	pub location:Location,

	/// The message of this related diagnostic information
	pub message:String,
}

/// Represents a text change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
	/// The range of the text document to be manipulated
	pub range:Range,

	/// The new text for the provided range
	pub new_text:String,
}

impl TextEdit {
	/// Create a new text edit
	pub fn new(range:Range, new_text:String) -> Self { Self { range, new_text } }

	/// Delete the text at the given range
	pub fn delete(range:Range) -> Self { Self { range, new_text:String::new() } }

	/// Insert the given text at the given position
	pub fn insert(position:Position, new_text:String) -> Self { Self { range:Range::empty(position), new_text } }

	/// Replace the text at the given range with the given text
	pub fn replace(range:Range, new_text:String) -> Self { Self { range, new_text } }
}

/// Workspace edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEdit {
	/// Holds changes to existing resources
	pub changes:Option<std::collections::HashMap<String, Vec<TextEdit>>>,
}

impl WorkspaceEdit {
	/// Create a new workspace edit
	pub fn new() -> Self { Self { changes:Some(std::collections::HashMap::new()) } }
}

impl Default for WorkspaceEdit {
	fn default() -> Self { Self::new() }
}

/// Completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
	/// The label of this completion item
	pub label:String,

	/// The kind of this completion item
	pub kind:Option<CompletionItemKind>,

	/// A human-readable string with additional information
	pub detail:Option<String>,

	/// A human-readable string that represents a doc-comment
	pub documentation:Option<CompletionItemDocumentation>,

	/// Select this item when showing
	#[serde(rename = "preselect")]
	pub preselect:Option<bool>,

	/// A string that should be used when comparing this item with other items
	pub sort_text:Option<String>,

	/// A string that should be used when filtering items
	pub filter_text:Option<String>,
}

impl CompletionItem {
	/// Create a new completion item
	pub fn new(label:String) -> Self {
		Self {
			label,
			kind:None,
			detail:None,
			documentation:None,
			preselect:None,
			sort_text:None,
			filter_text:None,
		}
	}
}

/// Completion item kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompletionItemKind {
	/// Text completion
	Text = 1,
	/// Method completion
	Method = 2,
	/// Function completion
	Function = 3,
	/// Constructor completion
	Constructor = 4,
	/// Field completion
	Field = 5,
	/// Variable completion
	Variable = 6,
	/// Class completion
	Class = 7,
	/// Interface completion
	Interface = 8,
	/// Module completion
	Module = 9,
	/// Property completion
	Property = 10,
	/// Unit completion
	Unit = 11,
	/// Value completion
	Value = 12,
	/// Enum completion
	Enum = 13,
	/// Keyword completion
	Keyword = 14,
	/// Snippet completion
	Snippet = 15,
	/// Color completion
	Color = 16,
	/// File completion
	File = 17,
	/// Reference completion
	Reference = 18,
	Folder = 19,
	EnumMember = 20,
	Constant = 21,
	Struct = 22,
	Event = 23,
	Operator = 24,
	TypeParameter = 25,
}

/// Completion item documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompletionItemDocumentation {
	/// Markdown string
	String(String),

	/// Value object
	Value(CompletionItemDocumentationValue),
}

/// Completion item documentation value object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItemDocumentationValue {
	/// The kind of documentation
	pub kind:String,

	/// The documentation content
	pub value:String,
}

/// Partial result token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialResultParams {
	/// An optional token that a server can use to report partial results
	#[serde(rename = "partialResultToken")]
	pub partial_result_token:Option<String>,
}

/// Work done progress params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDoneProgressParams {
	/// An optional token that a server can use to report work done progress
	#[serde(rename = "workDoneToken")]
	pub work_done_token:Option<String>,
}

/// Text document identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
	/// The text document's uri
	pub uri:String,
}

impl TextDocumentIdentifier {
	/// Create a new text document identifier
	pub fn new(uri:String) -> Self { Self { uri } }
}

/// Versioned text document identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
	/// The text document's uri
	pub uri:String,

	/// The version number of this document
	pub version:i32,
}

impl VersionedTextDocumentIdentifier {
	/// Create a new versioned text document identifier
	pub fn new(uri:String, version:i32) -> Self { Self { uri, version } }
}

/// Text document item
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextDocumentItem {
	/// The text document's uri
	pub uri:String,

	/// The text document's language identifier
	#[serde(rename = "languageId")]
	pub language_id:String,

	/// The version number of this document
	pub version:i32,

	/// The content of the opened text document
	pub text:String,
}

impl TextDocumentItem {
	/// Create a new text document item
	pub fn new(uri:String, language_id:String, version:i32, text:String) -> Self {
		Self { uri, language_id, version, text }
	}
}

/// The parameters sent in notifications/requests for user-initiated creation of
/// files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFilesParams {
	/// An array of all files/folders created in this operation
	pub files:Vec<FileCreate>,
}

/// Represents information to create a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreate {
	/// A file or folder uri
	pub uri:String,

	/// Additional options
	#[serde(rename = "options")]
	pub options:Option<CreateFileOptions>,
}

/// Options when creating a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileOptions {
	/// Overwrite existing file
	#[serde(rename = "overwrite")]
	pub overwrite:Option<bool>,

	/// Ignore if exists
	#[serde(rename = "ignoreIfExists")]
	pub ignore_if_exists:Option<bool>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_position() {
		let pos = Position::new(5, 10);
		assert_eq!(pos.line, 5);
		assert_eq!(pos.character, 10);

		let default = Position::default();
		assert_eq!(default.line, 0);
		assert_eq!(default.character, 0);
	}

	#[test]
	fn test_range() {
		let start = Position::new(0, 0);
		let end = Position::new(5, 10);
		let range = Range::new(start, end);

		assert!(range.contains(Position::new(3, 5)));
		assert!(!range.contains(Position::new(6, 0)));
	}

	#[test]
	fn test_text_edit_operations() {
		let range = Range::new(Position::new(0, 0), Position::new(0, 5));

		let replace = TextEdit::replace(range, "new text".to_string());
		assert_eq!(replace.new_text, "new text");

		let delete = TextEdit::delete(range);
		assert_eq!(delete.new_text, "");

		let insert = TextEdit::insert(Position::new(0, 0), "inserted".to_string());
		assert_eq!(insert.new_text, "inserted");
	}

	#[test]
	fn test_completion_item() {
		let item = CompletionItem::new("testFunction".to_string());
		assert_eq!(item.label, "testFunction");
	}
}
