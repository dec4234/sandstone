use std::fmt::{Debug, Display};
use std::io;

use thiserror::Error;

use crate::protocol::serialization::serializer_error::SerializingErr;

/// Any sort of error that could occur while performing or processing a network request.
#[derive(Error, Debug)]
pub enum NetworkError {
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
	#[error("Expected different packet: {0}")]
	ExpectedDifferentPacket(String),
	
	#[error(transparent)]
	SerializingErr(#[from] SerializingErr),
	#[error(transparent)]
	IOError(#[from] io::Error),
}