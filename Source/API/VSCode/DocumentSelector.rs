use serde::{Deserialize, Serialize};

/// Document filter
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
