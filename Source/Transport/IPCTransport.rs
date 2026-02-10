//! IPC Transport Implementation
//!
//! Provides inter-process communication (IPC) for Grove.
//! Supports Unix domain sockets on macOS/Linux.

use std::{
	path::{Path, PathBuf},
	sync::Arc,
};

use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::Transport::TransportStrategy;
use crate::Transport::TransportType;
use crate::Transport::TransportStats;
use crate::Transport::TransportConfig;

/// IPC transport for local process communication
#[derive(Clone, Debug)]
pub struct IPCTransportImpl {
	/// Socket path
	socket_path:Option<PathBuf>,
	/// Named pipe name (Windows)
	pipe_name:Option<String>,
	/// Transport configuration
	config:TransportConfig,
	/// Connection state
	connected:Arc<RwLock<bool>>,
	/// Transport statistics
	stats:Arc<RwLock<TransportStats>>,
}

impl IPCTransportImpl {
	/// Create a new IPC transport with default socket path
	pub fn new() -> anyhow::Result<Self> {
		#[cfg(unix)]
		{
			let socket_path = Self::default_socket_path();
			Ok(Self {
				socket_path:Some(socket_path),
				pipe_name:None,
				config:TransportConfig::default(),
				connected:Arc::new(RwLock::new(false)),
				stats:Arc::new(RwLock::new(TransportStats::default())),
			})
		}

		#[cfg(windows)]
		{
			Ok(Self {
				socket_path:None,
				pipe_name:Some(r"\\.\pipe\grove-ipc".to_string()),
				config:TransportConfig::default(),
				connected:Arc::new(RwLock::new(false)),
				stats:Arc::new(RwLock::new(TransportStats::default())),
			})
		}

		#[cfg(not(any(unix, windows)))]
		{
			Err(anyhow::anyhow!("IPC transport not supported on this platform"))
		}
	}

	/// Create a new IPC transport with a custom socket path
	///
	/// # Arguments
	///
	/// * `socket_path` - Path to the Unix domain socket
	///
	/// # Example
	///
	/// ```rust,no_run
	/// use std::path::Path;
	///
	/// use grove::Transport::IPCTransport;
	///
	/// # #[cfg(unix)]
	/// let transport = IPCTransport::with_socket_path(Path::new("/tmp/grove.sock"))?;
	/// # Ok::<(), anyhow::Error>(())
	/// ```
	pub fn with_socket_path<P:AsRef<Path>>(socket_path:P) -> anyhow::Result<Self> {
		#[cfg(unix)]
		{
			Ok(Self {
				socket_path:Some(socket_path.as_ref().to_path_buf()),
				pipe_name:None,
				config:TransportConfig::default(),
				connected:Arc::new(RwLock::new(false)),
				stats:Arc::new(RwLock::new(TransportStats::default())),
			})
		}

		#[cfg(not(unix))]
		{
			Err(anyhow::anyhow!("Unix sockets not supported on this platform"))
		}
	}

	/// Create a new IPC transport with a custom pipe name (Windows)
	#[cfg(windows)]
	pub fn with_pipe_name(pipe_name:&str) -> anyhow::Result<Self> {
		Ok(Self {
			socket_path:None,
			pipe_name:Some(pipe_name.to_string()),
			config:TransportConfig::default(),
			connected:Arc::new(RwLock::new(false)),
			stats:Arc::new(RwLock::new(TransportStats::default())),
		})
	}

	/// Create a new IPC transport with custom configuration
	pub fn with_config(config:TransportConfig) -> anyhow::Result<Self> {
		#[cfg(unix)]
		{
			let socket_path = Self::default_socket_path();
			Ok(Self {
				socket_path:Some(socket_path),
				pipe_name:None,
				config,
				connected:Arc::new(RwLock::new(false)),
				stats:Arc::new(RwLock::new(TransportStats::default())),
			})
		}

		#[cfg(windows)]
		{
			Ok(Self {
				socket_path:None,
				pipe_name:Some(r"\\.\pipe\grove-ipc".to_string()),
				config,
				connected:Arc::new(RwLock::new(false)),
				stats:Arc::new(RwLock::new(TransportStats::default())),
			})
		}

		#[cfg(not(any(unix, windows)))]
		{
			Err(anyhow::anyhow!("IPC transport not supported on this platform"))
		}
	}

	/// Get the default socket path for the current platform
	#[cfg(unix)]
	fn default_socket_path() -> PathBuf {
		let mut path = std::env::temp_dir();
		path.push("grove-ipc.sock");
		path
	}

	/// Get the socket path (Unix only)
	#[cfg(unix)]
	pub fn socket_path(&self) -> Option<&Path> { self.socket_path.as_deref() }

	/// Get the pipe name (Windows only)
	#[cfg(windows)]
	pub fn pipe_name(&self) -> Option<&str> { self.pipe_name.as_deref() }

	/// Get transport statistics
	pub async fn stats(&self) -> TransportStats { self.stats.read().await.clone() }

	/// Clean up the socket file if it exists
	#[cfg(unix)]
	async fn cleanup_socket(&self) -> anyhow::Result<()> {
		if let Some(path) = &self.socket_path {
			if path.exists() {
				tokio::fs::remove_file(path)
					.await
					.map_err(|e| anyhow::anyhow!("Failed to remove socket: {}", e))?;
				debug!("Removed existing socket: {:?}", path);
			}
		}
		Ok(())
	}
}

#[async_trait]
impl TransportStrategy for IPCTransportImpl {
	type Error = IPCTransportError;

	#[instrument(skip(self))]
	async fn connect(&self) -> Result<(), Self::Error> {
		info!("Connecting to IPC transport");

		#[cfg(unix)]
		{
			self.cleanup_socket()
				.await
				.map_err(|e| IPCTransportError::ConnectionFailed(e.to_string()))?;

			// For a complete implementation, we would create the Unix socket here
			// For now, we just mark as connected
			*self.connected.write().await = true;

			info!("IPC connection established: {:?}", self.socket_path);
		}

		#[cfg(windows)]
		{
			*self.connected.write().await = true;
			info!("IPC connection established: {:?}", self.pipe_name);
		}

		#[cfg(not(any(unix, windows)))]
		{
			return Err(IPCTransportError::NotSupported);
		}

		Ok(())
	}

	#[instrument(skip(self, request))]
	async fn send(&self, request:&[u8]) -> Result<Vec<u8>, Self::Error> {
		if !self.is_connected() {
			return Err(IPCTransportError::NotConnected);
		}

		debug!("Sending IPC request ({} bytes)", request.len());

		// For a complete implementation, this would send the request via the
		// socket/pipe For now, we return a mock response
		let response = vec![];

		// Update statistics
		let mut stats = self.stats.write().await;
		stats.record_sent(request.len() as u64, 0);
		stats.record_received(response.len() as u64);

		Ok(response)
	}

	#[instrument(skip(self, data))]
	async fn send_no_response(&self, data:&[u8]) -> Result<(), Self::Error> {
		if !self.is_connected() {
			return Err(IPCTransportError::NotConnected);
		}

		debug!("Sending IPC request without response ({} bytes)", data.len());

		// For a complete implementation, this would send the data via the socket/pipe
		// For now, we just update statistics
		let mut stats = self.stats.write().await;
		stats.record_sent(data.len() as u64, 0);

		Ok(())
	}

	#[instrument(skip(self))]
	async fn close(&self) -> Result<(), Self::Error> {
		info!("Closing IPC connection");

		*self.connected.write().await = false;

		#[cfg(unix)]
		{
			// Clean up socket file
			if let Some(path) = &self.socket_path {
				if path.exists() {
					tokio::fs::remove_file(path).await.map_err(|e| {
						warn!("Failed to remove socket: {}", e);
						// Don't fail the close operation on cleanup failure
						IPCTransportError::CleanupFailed(e.to_string())
					})?;
				}
			}
		}

		info!("IPC connection closed");

		Ok(())
	}

	fn is_connected(&self) -> bool { self.connected.blocking_read().to_owned() }

	fn transport_type(&self) -> TransportType {
		TransportType::IPC
	}
}

/// IPC transport errors
#[derive(Debug, thiserror::Error)]
pub enum IPCTransportError {
	#[error("Connection failed: {0}")]
	ConnectionFailed(String),

	#[error("Send failed: {0}")]
	SendFailed(String),

	#[error("Receive failed: {0}")]
	ReceiveFailed(String),

	#[error("Not connected")]
	NotConnected,

	#[error("IPC not supported on this platform")]
	NotSupported,

	#[error("Cleanup failed: {0}")]
	CleanupFailed(String),

	#[error("Socket error: {0}")]
	SocketError(String),

	#[error("Timeout")]
	Timeout,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Transport::Strategy::TransportStrategy;

	#[test]
	fn test_ipc_transport_creation() {
		#[cfg(any(unix, windows))]
		{
			let result = IPCTransportImpl::new();
			assert!(result.is_ok());
		}
	}

	#[test]
	fn test_ipc_transport_not_supported() {
		#[cfg(not(any(unix, windows)))]
		{
			let result = IPCTransportImpl::new();
			assert!(result.is_err());
		}
	}

	#[cfg(unix)]
	#[test]
	fn test_ipc_transport_with_socket_path() {
		let result = IPCTransportImpl::with_socket_path(Path::new("/tmp/test.sock"));
		assert!(result.is_ok());
		let transport = result.unwrap();
		assert_eq!(transport.socket_path(), Some(Path::new("/tmp/test.sock")));
	}

	#[cfg(windows)]
	#[test]
	fn test_ipc_transport_with_pipe_name() {
		let result = IPCTransportImpl::with_pipe_name(r"\\.\pipe\test");
		assert!(result.is_ok());
		let transport = result.unwrap();
		assert_eq!(transport.pipe_name(), Some(r"\\.\pipe\test"));
	}

	#[tokio::test]
	async fn test_ipc_transport_not_connected() {
		#[cfg(any(unix, windows))]
		{
			let transport = IPCTransportImpl::new().unwrap();
			assert!(!transport.is_connected());
		}
	}
}
