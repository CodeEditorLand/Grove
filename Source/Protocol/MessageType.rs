//! Message types for Spine protocol.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageType {
	Heartbeat = 0,

	Register = 1,

	Unregister = 2,

	Event = 3,

	Request = 4,

	Response = 5,

	Error = 6,
}

impl MessageType {
	pub fn as_u32(self) -> u32 { self as u32 }

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
