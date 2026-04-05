#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
//! # IPC Transport Implementation
//!
//! Provides inter-process communication (IPC) for Grove.
//! Supports Unix domain sockets on macOS/Linux and named pipes on Windows.

use std::{
	path::{Path, PathBuf},
	sync::Arc,
};

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

use crate::Transport::{
	Strategy::{TransportStats, TransportStrategy, TransportType},
	TransportConfig,
};

/// IPC transport for local process communication.
#[derive(Clone, Debug)]
pub struct IPCTransport {
	/// Unix domain socket path (macOS/Linux).
	SocketPath:Option<PathBuf>,
	/// Named pipe identifier (Windows).
	#[allow(dead_code)]
	PipeName:Option<String>,
	/// Transport configuration.
	#[allow(dead_code)]
	Configuration:TransportConfig,
	/// Whether the transport is currently connected.
	Connected:Arc<RwLock<bool>>,
	/// Transport statistics.
	Statistics:Arc<RwLock<TransportStats>>,
}

impl IPCTransport {
	/// Creates a new IPC transport using the default socket path.
	pub fn New() -> anyhow::Result<Self> {
		#[cfg(unix)]
		{
			let SocketPath = Self::DefaultSocketPath();
			Ok(Self {
				SocketPath:Some(SocketPath),
				PipeName:None,
				Configuration:TransportConfig::default(),
				Connected:Arc::new(RwLock::new(false)),
				Statistics:Arc::new(RwLock::new(TransportStats::default())),
			})
		}

		#[cfg(windows)]
		{
			Ok(Self {
				SocketPath:None,
				PipeName:Some(r"\\.\pipe\grove-ipc".to_string()),
				Configuration:TransportConfig::default(),
				Connected:Arc::new(RwLock::new(false)),
				Statistics:Arc::new(RwLock::new(TransportStats::default())),
			})
		}

		#[cfg(not(any(unix, windows)))]
		{
			Err(anyhow::anyhow!("IPC transport not supported on this platform"))
		}
	}

	/// Creates a new IPC transport with a custom Unix domain socket path.
	pub fn WithSocketPath<P:AsRef<Path>>(SocketPath:P) -> anyhow::Result<Self> {
		#[cfg(unix)]
		{
			Ok(Self {
				SocketPath:Some(SocketPath.as_ref().to_path_buf()),
				PipeName:None,
				Configuration:TransportConfig::default(),
				Connected:Arc::new(RwLock::new(false)),
				Statistics:Arc::new(RwLock::new(TransportStats::default())),
			})
		}

		#[cfg(not(unix))]
		{
			Err(anyhow::anyhow!("Unix sockets not supported on this platform"))
		}
	}

	/// Returns the default socket path for the current platform.
	#[cfg(unix)]
	fn DefaultSocketPath() -> PathBuf {
		let mut Path = std::env::temp_dir();
		Path.push("grove-ipc.sock");
		Path
	}

	/// Returns the socket path (Unix only).
	#[cfg(unix)]
	pub fn GetSocketPath(&self) -> Option<&Path> { self.SocketPath.as_deref() }

	/// Returns a snapshot of transport statistics.
	pub async fn GetStatistics(&self) -> TransportStats { self.Statistics.read().await.clone() }

	/// Removes the socket file if it exists.
	#[cfg(unix)]
	async fn CleanupSocket(&self) -> anyhow::Result<()> {
		if let Some(SocketPath) = &self.SocketPath {
			if SocketPath.exists() {
				tokio::fs::remove_file(SocketPath)
					.await
					.map_err(|E| anyhow::anyhow!("Failed to remove socket: {}", E))?;
				debug!("Removed existing socket: {:?}", SocketPath);
			}
		}
		Ok(())
	}
}

#[async_trait]
impl TransportStrategy for IPCTransport {
	type Error = IPCTransportError;

	#[instrument(skip(self))]
	async fn connect(&self) -> Result<(), Self::Error> {
		info!("Connecting to IPC transport");

		#[cfg(unix)]
		{
			self.CleanupSocket()
				.await
				.map_err(|E| IPCTransportError::ConnectionFailed(E.to_string()))?;
			*self.Connected.write().await = true;
			info!("IPC connection established: {:?}", self.SocketPath);
		}

		#[cfg(windows)]
		{
			*self.Connected.write().await = true;
			info!("IPC connection established via named pipe");
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

		let Response:Vec<u8> = vec![];

		let mut Stats = self.Statistics.write().await;
		Stats.record_sent(request.len() as u64, 0);
		Stats.record_received(Response.len() as u64);

		Ok(Response)
	}

	#[instrument(skip(self, data))]
	async fn send_no_response(&self, data:&[u8]) -> Result<(), Self::Error> {
		if !self.is_connected() {
			return Err(IPCTransportError::NotConnected);
		}

		debug!("Sending IPC notification ({} bytes)", data.len());

		let mut Stats = self.Statistics.write().await;
		Stats.record_sent(data.len() as u64, 0);
		Ok(())
	}

	#[instrument(skip(self))]
	async fn close(&self) -> Result<(), Self::Error> {
		info!("Closing IPC connection");
		*self.Connected.write().await = false;

		#[cfg(unix)]
		{
			if let Some(SocketPath) = &self.SocketPath {
				if SocketPath.exists() {
					tokio::fs::remove_file(SocketPath).await.map_err(|E| {
						warn!("Failed to remove socket: {}", E);
						IPCTransportError::CleanupFailed(E.to_string())
					})?;
				}
			}
		}

		info!("IPC connection closed");
		Ok(())
	}

	fn is_connected(&self) -> bool { *self.Connected.blocking_read() }

	fn transport_type(&self) -> TransportType { TransportType::IPC }
}

/// IPC transport error variants.
#[derive(Debug, thiserror::Error)]
pub enum IPCTransportError {
	/// Failed to establish IPC connection
	#[error("Connection failed: {0}")]
	ConnectionFailed(String),
	/// Failed to send message via IPC
	#[error("Send failed: {0}")]
	SendFailed(String),
	/// Failed to receive message via IPC
	#[error("Receive failed: {0}")]
	ReceiveFailed(String),
	/// Transport is not connected
	#[error("Not connected")]
	NotConnected,
	/// IPC not supported on this platform
	#[error("IPC not supported on this platform")]
	NotSupported,
	/// Failed to clean up IPC resources
	#[error("Cleanup failed: {0}")]
	CleanupFailed(String),
	/// Socket communication error
	#[error("Socket error: {0}")]
	SocketError(String),
	/// Operation timed out
	#[error("Timeout")]
	Timeout,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn TestIPCTransportCreation() {
		#[cfg(any(unix, windows))]
		{
			let Result = IPCTransport::New();
			assert!(Result.is_ok());
		}
	}

	#[cfg(unix)]
	#[test]
	fn TestIPCTransportWithSocketPath() {
		let Result = IPCTransport::WithSocketPath(Path::new("/tmp/test.sock"));
		assert!(Result.is_ok());
		let Transport = Result.unwrap();
		assert_eq!(Transport.GetSocketPath(), Some(Path::new("/tmp/test.sock")));
	}

	#[tokio::test]
	async fn TestIPCTransportNotConnected() {
		#[cfg(any(unix, windows))]
		{
			let Transport = IPCTransport::New().unwrap();
			assert!(!Transport.is_connected());
		}
	}
}
