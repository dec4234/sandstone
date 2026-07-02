//! Network error types.

use std::fmt::Debug;
use std::io;

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::util::encryption::EncryptionError;
use thiserror::Error;

/// Any sort of error that could occur while performing or processing a network request.
#[derive(Error, Debug)]
pub enum NetworkError {
	/// Like when using try_receive and it returns None.
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
