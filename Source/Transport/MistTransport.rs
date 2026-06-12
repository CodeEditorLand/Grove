//! WebSocket transport implemented over Mist's JSON-RPC-over-WS protocol.
//!
//! `MistTransport` wraps [`Mist::WebSocket::Client`] and exposes it through
//! Grove's [`TransportStrategy`] interface so any Grove consumer can switch
//! from gRPC/IPC to the local WebSocket channel with a single variant swap.
//!
//! # Wire protocol
//!
//! `send(bytes)` expects the bytes to deserialise as
//! `{ "method": "<string>", "params": <json> }`. The result is the serialised
//! JSON value returned by the server.
//!
//! `send_no_response(bytes)` uses the same input shape but sends a Mist
//! notification frame (`id: null`) - the server does not reply.
//!
//! # Feature gate
//!
//! This module is compiled only under the `websocket` cargo feature.

#[cfg(feature = "websocket")]
pub use websocket::MistTransport;

#[cfg(feature = "websocket")]
mod websocket {

	use std::sync::Arc;

	use async_trait::async_trait;
	use tokio::sync::Mutex;

	use crate::Transport::Strategy::{TransportStrategy, TransportType};

	/// Grove transport backed by the Mist WebSocket server running on
	/// Mountain. Connects to `ws://127.0.0.1:<port>` with the shared secret
	/// established at Mountain boot time.
	pub struct MistTransport {
		Address:String,

		Client:Arc<Mutex<Option<Arc<Mist::WebSocket::Client>>>>,
	}

	impl std::fmt::Debug for MistTransport {
		fn fmt(&self, Formatter:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			Formatter
				.debug_struct("MistTransport")
				.field("Address", &self.Address)
				.field("Connected", &self.is_connected())
				.finish()
		}
	}

	impl MistTransport {
		/// Creates a new `MistTransport` targeting `address`
		/// (e.g. `"ws://127.0.0.1:5051"`).
		pub fn New(Address:impl Into<String>) -> Self {
			Self { Address:Address.into(), Client:Arc::new(Mutex::new(None)) }
		}

		/// Returns the stored client if connected, else `None`.
		async fn GetClient(&self) -> Option<Arc<Mist::WebSocket::Client>> { self.Client.lock().await.clone() }

		/// Opens a fresh Mist client to `Address`, attaching the
		/// per-spawn shared secret from `MountainWebSocketSecret`
		/// (inherited from Mountain's environment) when present.
		async fn Dial(&self) -> anyhow::Result<Arc<Mist::WebSocket::Client>> {
			match std::env::var("MountainWebSocketSecret")
				.ok()
				.and_then(|Hex| Mist::WebSocket::SharedSecret::from_hex(&Hex).ok())
			{
				Some(Secret) => Mist::WebSocket::Client::ConnectWithSecret(&self.Address, &Secret).await,

				None => Mist::WebSocket::Client::connect(&self.Address).await,
			}
		}

		/// Reconnects with exponential backoff (100/200/400/1000/2000 ms,
		/// 5 attempts), storing and returning the fresh client. Propagates
		/// the last connect error once the attempts are exhausted.
		async fn Reconnect(&self) -> Result<Arc<Mist::WebSocket::Client>, MistTransportError> {
			const BackoffMilliseconds:[u64; 5] = [100, 200, 400, 1000, 2000];

			let mut LastError:Option<anyhow::Error> = None;

			for Delay in BackoffMilliseconds {
				tokio::time::sleep(std::time::Duration::from_millis(Delay)).await;

				match self.Dial().await {
					Ok(Client) => {
						*self.Client.lock().await = Some(Client.clone());

						return Ok(Client);
					},

					Err(Error) => LastError = Some(Error),
				}
			}

			Err(MistTransportError::ConnectionFailed(
				LastError.unwrap_or_else(|| anyhow::anyhow!("reconnect attempts exhausted")),
			))
		}

		/// Returns the live client, transparently reconnecting when the
		/// stored one has dropped its connection. A `None` slot means
		/// `connect()` was never called (or `close()` was) - that stays
		/// a hard `NotConnected` error rather than an implicit dial.
		async fn LiveClient(&self) -> Result<Arc<Mist::WebSocket::Client>, MistTransportError> {
			let Client = self.GetClient().await.ok_or(MistTransportError::NotConnected)?;

			if Client.is_closed() { self.Reconnect().await } else { Ok(Client) }
		}
	}

	/// Returns `true` when a Mist invoke/notify error string indicates a
	/// dropped connection (as opposed to a server-side handler error).
	fn IsDisconnect(Error:&str) -> bool {
		Error.contains("connection closed") || Error.contains("send failed") || Error.contains("request cancelled")
	}

	/// Transport error type for MistTransport.
	#[derive(Debug, thiserror::Error)]
	pub enum MistTransportError {
		#[error("not connected")]
		NotConnected,

		#[error("connection failed: {0}")]
		ConnectionFailed(anyhow::Error),

		#[error("request failed: {0}")]
		RequestFailed(String),

		#[error("json error: {0}")]
		Json(#[from] serde_json::Error),
	}

	#[async_trait]
	impl TransportStrategy for MistTransport {
		type Error = MistTransportError;

		async fn connect(&self) -> Result<(), Self::Error> {
			let Client = self.Dial().await.map_err(MistTransportError::ConnectionFailed)?;

			*self.Client.lock().await = Some(Client);

			Ok(())
		}

		async fn send(&self, Request:&[u8]) -> Result<Vec<u8>, Self::Error> {
			let Msg:serde_json::Value = serde_json::from_slice(Request)?;

			let Method = Msg.get("method").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Params = Msg.get("params").cloned().unwrap_or(serde_json::Value::Null);

			let Client = self.LiveClient().await?;

			let Result = match Client.invoke(&Method, Params.clone()).await {
				Ok(Result) => Result,

				Err(Error) if IsDisconnect(&Error) => {
					let Fresh = self.Reconnect().await?;

					Fresh.invoke(&Method, Params).await.map_err(MistTransportError::RequestFailed)?
				},

				Err(Error) => return Err(MistTransportError::RequestFailed(Error)),
			};

			Ok(serde_json::to_vec(&Result)?)
		}

		async fn send_no_response(&self, Data:&[u8]) -> Result<(), Self::Error> {
			let Msg:serde_json::Value = serde_json::from_slice(Data)?;

			let Method = Msg.get("method").and_then(|V| V.as_str()).unwrap_or("").to_string();

			let Params = Msg.get("params").cloned().unwrap_or(serde_json::Value::Null);

			let Client = self.LiveClient().await?;

			match Client.notify(&Method, Params.clone()).await {
				Ok(()) => Ok(()),

				Err(Error) if IsDisconnect(&Error) => {
					let Fresh = self.Reconnect().await?;

					Fresh.notify(&Method, Params).await.map_err(MistTransportError::RequestFailed)
				},

				Err(Error) => Err(MistTransportError::RequestFailed(Error)),
			}
		}

		async fn close(&self) -> Result<(), Self::Error> {
			*self.Client.lock().await = None;

			Ok(())
		}

		fn is_connected(&self) -> bool {
			self.Client
				.try_lock()
				.ok()
				.and_then(|G| G.as_ref().map(|C| !C.is_closed()))
				.unwrap_or(false)
		}

		fn transport_type(&self) -> TransportType { TransportType::WebSocket }
	}
}
