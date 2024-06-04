use std::array::TryFromSliceError;
use std::fmt::{Debug, Display};
use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializingErr {
	#[error("VarInt ended prematurely")]
	InvalidEndOfVarInt,
	#[error("The VarType did not end when it should have. {0}")]
	VarTypeTooLong(String),
	#[error(transparent)]
	CouldNotDeserializeString(#[from] Utf8Error),
	#[error(transparent)]
	StringFromSliceError(#[from] TryFromSliceError),
	#[error("Input ended prematurely")]
	InputEnded,
	#[error("There is unused input data left")]
	LeftoverInput,
	#[error("Unknown deserialization failure")]
	UnknownFailure,
	#[error("{0}")]
	UniqueFailure(String),
	#[error("The current packet state does not match what is needed to deserialize this packet")]
	InvalidPacketState,
}