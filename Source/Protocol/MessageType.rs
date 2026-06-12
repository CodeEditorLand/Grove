//! Protocol message types.

pub enum MessageType {
	/// Heartbeat message
	Heartbeat = 0,

	/// Registration message
	Register = 1,

	/// Unregistration message
	Unregister = 2,

	/// Event message
	Event = 3,

	/// Request message
	Request = 4,

	/// Response message
	Response = 5,

	/// Error message
	Error = 6,
}

impl MessageType {
	/// Convert to u32
	pub fn as_u32(self) -> u32 { self as u32 }

	/// Convert from u32
	pub fn from_u32(value:u32) -> Option<Self> {
		match value {
			0 => Some(Self::Heartbeat),

			1 => Some(Self::Register),

			2 => Some(Self::Unregister),

			3 => Some(Self::Event),

			4 => Some(Self::Request),

			5 => Some(Self::Response),

			6 => Some(Self::Error),

			_ => None,
		}
	}
}

/// Protocol error types
