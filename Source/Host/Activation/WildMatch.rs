//! Simple wildcard matching for flexible activation events

/// Simple wildcard matching for flexible activation events
pub(crate) struct WildMatch {
	pattern:String,
}

impl WildMatch {
	pub fn new(pattern:&str) -> Self { Self { pattern:pattern.to_lowercase() } }

	pub fn matches(&self, text:&str) -> bool {
		let text = text.to_lowercase();

		// Handle * wildcard
		if self.pattern == "*" {
			return true;
		}

		// Handle patterns starting with *
		if self.pattern.starts_with('*') {
			let suffix = &self.pattern[1..];

			return text.ends_with(suffix);
		}

		// Handle patterns ending with *
		if self.pattern.ends_with('*') {
			let prefix = &self.pattern[..self.pattern.len() - 1];

			return text.starts_with(prefix);
		}

		// Exact match
		self.pattern == text
	}
}
