#[cfg(test)]
mod tests {

	use crate::Host::Activation::{ActivationContext, ActivationEvent, WildMatch};

	#[test]
	fn test_activation_event_parsing() {
		let event = ActivationEvent::from_str("*").unwrap();

		assert_eq!(event, ActivationEvent::Star);

		let event = ActivationEvent::from_str("onCommand:test.command").unwrap();

		assert_eq!(event, ActivationEvent::Command("test.command".to_string()));

		let event = ActivationEvent::from_str("onLanguage:rust").unwrap();

		assert_eq!(event, ActivationEvent::Language("rust".to_string()));
	}

	#[test]
	fn test_activation_event_to_string() {
		assert_eq!(ActivationEvent::Star.to_string(), "*");

		assert_eq!(ActivationEvent::Command("test".to_string()).to_string(), "onCommand:test");

		assert_eq!(ActivationEvent::Language("rust".to_string()).to_string(), "onLanguage:rust");
	}

	#[test]
	fn test_activation_context_default() {
		let context = ActivationContext::default();

		assert!(context.workspace_path.is_none());

		assert!(context.current_file.is_none());

		assert!(!context.active_editor);
	}

	#[test]
	fn test_wildcard_matching() {
		let matcher = WildMatch::new("*");

		assert!(matcher.matches("anything"));

		let matcher = WildMatch::new("prefix*");

		assert!(matcher.matches("prefix_suffix"));

		assert!(!matcher.matches("noprefix_suffix"));

		let matcher = WildMatch::new("*suffix");

		assert!(matcher.matches("prefix_suffix"));

		assert!(!matcher.matches("prefix_suffix_not"));
	}
}
