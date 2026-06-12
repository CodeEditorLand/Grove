use std::sync::{
	Arc,
	atomic::{AtomicU32, Ordering},
};

use crate::dev_log;
use super::{Disposable::Disposable, DocumentSelector::DocumentSelector};

/// Tracks all active language provider registrations with their handles.
///
/// A registration is added when an extension calls `register_*_provider` and
/// removed when `Disposable::dispose()` is called on the returned handle.
#[derive(Debug, Default)]
pub struct ProviderStore {
	/// Map from handle → (provider_type, selector) for diagnostics.
	pub entries:std::sync::Mutex<std::collections::HashMap<u32, (String, String)>>,

	/// Monotonically increasing handle counter.
	pub next_handle:AtomicU32,
}

impl ProviderStore {
	/// Returns the next unique handle and inserts a registration record.
	pub fn insert(&self, provider_type:&str, selector:&str) -> u32 {
		let Handle = self.next_handle.fetch_add(1, Ordering::Relaxed);

		if let Ok(mut Guard) = self.entries.lock() {
			Guard.insert(Handle, (provider_type.to_string(), selector.to_string()));
		}

		Handle
	}

	/// Removes a registration by handle (called from Disposable::dispose).
	pub fn remove(&self, handle:u32) {
		if let Ok(mut Guard) = self.entries.lock() {
			Guard.remove(&handle);
		}
	}

	/// Returns the number of active registrations.
	pub fn len(&self) -> usize { self.entries.lock().map(|G| G.len()).unwrap_or(0) }
}
