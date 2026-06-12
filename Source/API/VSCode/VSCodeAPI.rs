use std::sync::Arc;

use crate::{API::Types::*, Transport::Strategy::Transport};
use super::{
	CommandNamespace::CommandNamespace,
	Env::Env,
	ExtensionNamespace::ExtensionNamespace,
	LanguageNamespace::LanguageNamespace,
	Window::Window,
	Workspace::Workspace,
};

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
