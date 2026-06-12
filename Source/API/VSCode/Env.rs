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
