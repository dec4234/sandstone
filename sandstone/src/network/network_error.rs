//! Network error types.

use std::fmt::Debug;
use std::io;

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::util::encryption::EncryptionError;
use thiserror::Error;

/// Any sort of error that could occur while performing or processing a network request.
#[derive(Error, Debug)]
pub enum NetworkError {
	/// The peer closed the connection: a read returned 0 bytes, or the stream ended before a full
	/// packet was received.
	#[error("No data received from stream")]
	NoDataReceived,
	#[error("Connection aborted locally")]
	ConnectionAbortedLocally,
	#[error("Connection aborted remotely")]
	ConnectionAbortedRemotely,
	#[error("Invalid packet state")]
	InvalidPacketState,
	#[error("{0}")]
	InvalidNextState(String),
	#[error("Invalid packet direction")]
	InvalidPacketDirection,
	#[error("Packet too large")]
	PacketTooLarge,
	/// Peeking reads bytes without consuming them, which cannot work once encryption is
	/// enabled: decrypting the peeked bytes advances the CFB8 cipher state that the next real
	/// read depends on, desynchronizing the stream.
	#[error("Cannot peek packets once encryption is enabled")]
	PeekNotSupportedWhileEncrypted,
	#[error("Expected different packet: {0}")]
	ExpectedDifferentPacket(String),
	#[error(transparent)]
	SerializingErr(#[from] SerializingErr),
	#[error(transparent)]
	IOError(#[from] io::Error),
	#[error(transparent)]
	EncryptionError(#[from] EncryptionError),
	#[error(transparent)]
	HttpError(#[from] mojang_api::http::HttpError)
}

impl PartialEq for NetworkError {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(NetworkError::NoDataReceived, NetworkError::NoDataReceived) => true,
			(NetworkError::ConnectionAbortedLocally, NetworkError::ConnectionAbortedLocally) => true,
			(NetworkError::ConnectionAbortedRemotely, NetworkError::ConnectionAbortedRemotely) => true,
			(NetworkError::InvalidPacketState, NetworkError::InvalidPacketState) => true,
			(NetworkError::InvalidNextState(a), NetworkError::InvalidNextState(b)) => a == b,
			(NetworkError::InvalidPacketDirection, NetworkError::InvalidPacketDirection) => true,
			(NetworkError::PacketTooLarge, NetworkError::PacketTooLarge) => true,
			(NetworkError::PeekNotSupportedWhileEncrypted, NetworkError::PeekNotSupportedWhileEncrypted) => true,
			(NetworkError::ExpectedDifferentPacket(a), NetworkError::ExpectedDifferentPacket(b)) => a == b,

			(NetworkError::SerializingErr(a), NetworkError::SerializingErr(b)) => a == b,
			(NetworkError::IOError(a), NetworkError::IOError(b)) => a.to_string() == b.to_string(),
			(NetworkError::EncryptionError(a), NetworkError::EncryptionError(b)) => a.to_string() == b.to_string(),
			_ => false,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::NetworkError;
	use crate::protocol::serialization::serializer_error::SerializingErr;
	use std::io;

	/// Wrapped errors are `transparent`: the text a user sees is the inner error's text, with no
	/// extra prefix. Variants with a payload show the payload.
	#[test]
	fn display_messages() {
		assert_eq!(NetworkError::PacketTooLarge.to_string(), "Packet too large");
		assert_eq!(NetworkError::InvalidNextState("7".into()).to_string(), "7");
		assert_eq!(NetworkError::ExpectedDifferentPacket("Ping".into()).to_string(), "Expected different packet: Ping");
		assert_eq!(NetworkError::from(SerializingErr::InputEnded).to_string(), SerializingErr::InputEnded.to_string());
		let io_err = io::Error::other("boom");
		assert_eq!(NetworkError::from(io_err).to_string(), "boom");
	}

	/// IO errors are not comparable, so equality uses the message. Different variants are never equal.
	#[test]
	fn eq_compares_payloads() {
		assert_eq!(NetworkError::from(SerializingErr::InputEnded), NetworkError::from(SerializingErr::InputEnded));
		assert_ne!(NetworkError::from(SerializingErr::InputEnded), NetworkError::from(SerializingErr::LeftoverInput));
		assert_eq!(NetworkError::from(io::Error::other("a")), NetworkError::from(io::Error::other("a")));
		assert_ne!(NetworkError::from(io::Error::other("a")), NetworkError::from(io::Error::other("b")));
		assert_ne!(NetworkError::NoDataReceived, NetworkError::PacketTooLarge);
	}
}
