//! VS Code API Facade Module
//!
//! Provides the VS Code API facade for Grove extensions.
//! This implements the interface described in vscode.d.ts for extension
//! compatibility.

pub mod CommandNamespace;

pub mod CompletionItemProvider;

pub mod DiagnosticCollection;

pub mod Disposable;

pub mod DocumentSelector;

pub mod Env;

pub mod ExtensionNamespace;

pub mod LanguageNamespace;

pub mod OutputChannel;

pub mod ProviderStore;

pub mod VSCodeAPI;

pub mod Window;

pub mod Workspace;

pub mod WorkspaceConfiguration;

#[cfg(test)]
mod Tests;

// Re-exports for convenience
pub use CommandNamespace::*;
pub use CompletionItemProvider::*;
pub use DiagnosticCollection::*;
pub use Disposable::*;
pub use DocumentSelector::*;
pub use Env::*;
pub use ExtensionNamespace::*;
pub use LanguageNamespace::*;
pub use OutputChannel::*;
pub use ProviderStore::*;
pub use VSCodeAPI::*;
pub use Window::*;
pub use Workspace::*;
pub use WorkspaceConfiguration::*;
