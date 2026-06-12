//! Factory functions for creating transport instances.

use anyhow::Result;

use super::{IPCTransport, Strategy, WASMTransport, gRPCTransport};

/// Creates the default transport (gRPC to localhost).
pub fn CreateDefaultTransport() -> Strategy::Transport { Strategy::Transport::default() }

/// Creates a gRPC transport connecting to the given address.
pub fn CreategRPCTransport(Address:&str) -> Result<Strategy::Transport> {
	Ok(Strategy::Transport::gRPC(gRPCTransport::gRPCTransport::New(Address)?))
}

/// Creates an IPC transport using the default socket/pipe path.
pub fn CreateIPCTransport() -> Result<Strategy::Transport> {
	Ok(Strategy::Transport::IPC(IPCTransport::IPCTransport::New()?))
}

/// Creates a WebSocket transport connecting to the Mist server at `address`.
///
/// Requires the `websocket` cargo feature. The returned transport must be
/// connected with `.connect().await` before use.
#[cfg(feature = "websocket")]
pub fn CreateWebSocketTransport(Address:&str) -> Result<Strategy::Transport> {
	use super::MistTransport;

	Ok(Strategy::Transport::WebSocket(MistTransport::MistTransport::New(Address)))
}

/// Creates a WASM transport with the given configuration.
pub fn CreateWASMTransport(
	EnableWASI:bool,

	MemoryLimitMegabytes:u64,

	MaxExecutionTimeMilliseconds:u64,
) -> Result<Strategy::Transport> {
	Ok(Strategy::Transport::WASM(WASMTransport::WASMTransportImpl::new(
		EnableWASI,
		MemoryLimitMegabytes,
		MaxExecutionTimeMilliseconds,
	)?))
}
