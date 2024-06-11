use std::array::TryFromSliceError;
use std::fmt::{Debug, Display};
use std::string::FromUtf8Error;

use thiserror::Error;

/*
The purpose of this file is to describe the SerializingError type, which is used to represent errors
for serialization and deserialization operations. This is useful for debugging and error handling.
 */

/// A type that describes common errors encountered while serializing or deserializing network data.
/// Each error either provides a description of the error or transparently passes the internal error,
/// usually another error type.
#[derive(Error, Debug, Clone)]
pub enum SerializingErr {
	#[error("VarInt ended prematurely")]
	InvalidEndOfVarInt,
	#[error("The VarType did not end when it should have. {0}")]
	VarTypeTooLong(String),
	#[error(transparent)]
	CouldNotDeserializeString(#[from] FromUtf8Error),
	#[error(transparent)]
	StringFromSliceError(#[from] TryFromSliceError),
	#[error("Input ended prematurely")]
	InputEnded,
	#[error("Out of bounds")]
	OutOfBounds,
	#[error("There is unused input data left")]
	LeftoverInput,
	#[error("Unknown deserialization failure")]
	UnknownFailure,
	#[error("{0}")]
	UniqueFailure(String),
	#[error("The current packet state does not match what is needed to deserialize this packet")]
	InvalidPacketState,
}

impl PartialEq for SerializingErr {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::InvalidEndOfVarInt, Self::InvalidEndOfVarInt) => true,
			(Self::VarTypeTooLong(a), Self::VarTypeTooLong(b)) => a == b,
			(Self::CouldNotDeserializeString(a), Self::CouldNotDeserializeString(b)) => a == b,
			(Self::StringFromSliceError(a), Self::StringFromSliceError(b)) => a.to_string() == b.to_string(),
			(Self::InputEnded, Self::InputEnded) => true,
			(Self::OutOfBounds, Self::OutOfBounds) => true,
			(Self::LeftoverInput, Self::LeftoverInput) => true,
			(Self::UnknownFailure, Self::UnknownFailure) => true,
			(Self::UniqueFailure(a), Self::UniqueFailure(b)) => a == b,
			(Self::InvalidPacketState, Self::InvalidPacketState) => true,
			_ => false,
		}
	}
}