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
	#[error("Could not deserialize String")]
	CouldNotDeserializeString,
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

impl From<Utf8Error> for SerializingErr {
	fn from(value: Utf8Error) -> Self {
		Self::CouldNotDeserializeString
	}
}

impl From<TryFromSliceError> for SerializingErr {
	fn from(value: TryFromSliceError) -> Self {
		Self::UniqueFailure("Something went wrong when converting from bytes to primitive".to_string())
	}
}