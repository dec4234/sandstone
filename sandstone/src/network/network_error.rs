use std::fmt::{Debug, Display};
use std::io;

use thiserror::Error;

use crate::packets::serialization::serializer_error::SerializingErr;

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
	Other(#[from] io::Error),
}