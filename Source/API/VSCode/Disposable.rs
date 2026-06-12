/// Disposable resource handle.
///
/// Returned by all `register_*_provider` methods. Calling `dispose()` removes
/// the provider registration from the `LanguageNamespace` store.
pub struct Disposable {
	callback:Option<Box<dyn FnOnce() + Send + Sync>>,
}

impl std::fmt::Debug for Disposable {
	fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Disposable")
			.field("has_callback", &self.callback.is_some())
			.finish()
	}
}

impl Clone for Disposable {
	/// Cloning a Disposable produces a no-op copy.
	/// The original disposable retains the callback.
	fn clone(&self) -> Self { Self { callback:None } }
}

impl Disposable {
	/// Create a no-op disposable.
	pub fn new() -> Self { Self { callback:None } }

	/// Create a disposable with a callback invoked on `dispose()`.
	pub fn with_callback(callback:Box<dyn FnOnce() + Send + Sync>) -> Self { Self { callback:Some(callback) } }

	/// Dispose the resource, invoking the registered callback if present.
	pub fn dispose(mut self) {
		if let Some(Callback) = self.callback.take() {
			Callback();
		}
	}
}

impl Default for Disposable {
	fn default() -> Self { Self::new() }
}
