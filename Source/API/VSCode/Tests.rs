#[cfg(test)]
mod tests {

	use super::super::*;

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
