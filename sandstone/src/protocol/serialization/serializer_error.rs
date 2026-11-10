//! This file describes the SerializingError type, which is used to represent errors
//! for serialization and deserialization operations. This is useful for debugging and error handling.

use std::array::TryFromSliceError;
use std::fmt::Debug;
use std::string::FromUtf8Error;

use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
use thiserror::Error;

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
	#[error("There is unused input data left")]
	LeftoverInput,
	#[error("Unknown deserialization failure")]
	UnknownFailure,
	#[error("{0}")]
	UniqueFailure(String),
	#[error("The current packet state does not match what is needed to deserialize this packet")]
	InvalidPacketState,
	#[error("Not a valid bits per entry")]
	InvalidBitsPerEntry,
	#[error("The input data is not a valid enum value: {0}")]
	InvalidEnumValue(i8),
	#[error("Nbt input is missing a field: {0}")]
	MissingField(String),
	#[error("Failed to find packet: {0}")]
	NoKnownPacket(String),
	#[error("Failed to deserialize: {0}")]
	DeserializationError(String),
	#[error(transparent)]
	NbtError(#[from] NbtError),
	#[error("The input data is inconsistent with another field: {0}")]
	InconsistentField(String),
	#[error("The deserialized data is not valid for the range of possible values: {0}")]
	OutOfBounds(String),
	#[error("Failed to serialize: {0}")]
	FailedToSerializeJSON(String),
}

impl PartialEq for SerializingErr {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::InvalidEndOfVarInt, Self::InvalidEndOfVarInt) => true,
			(Self::VarTypeTooLong(a), Self::VarTypeTooLong(b)) => a == b,
			(Self::CouldNotDeserializeString(a), Self::CouldNotDeserializeString(b)) => a == b,
			(Self::StringFromSliceError(a), Self::StringFromSliceError(b)) => a.to_string() == b.to_string(),
			(Self::InputEnded, Self::InputEnded) => true,
			(Self::LeftoverInput, Self::LeftoverInput) => true,
			(Self::UnknownFailure, Self::UnknownFailure) => true,
			(Self::UniqueFailure(a), Self::UniqueFailure(b)) => a == b,
			(Self::InvalidPacketState, Self::InvalidPacketState) => true,
			(Self::InconsistentField(a), Self::InconsistentField(b)) => a == b,
			(Self::OutOfBounds(a), Self::OutOfBounds(b)) => a == b,
			(Self::InvalidBitsPerEntry, Self::InvalidBitsPerEntry) => true,
			(Self::InvalidEnumValue(a), Self::InvalidEnumValue(b)) => a == b,
			(Self::MissingField(a), Self::MissingField(b)) => a == b,
			(Self::NoKnownPacket(a), Self::NoKnownPacket(b)) => a == b,
			(Self::DeserializationError(a), Self::DeserializationError(b)) => a == b,
			(Self::NbtError(a), Self::NbtError(b)) => a == b,
			(Self::FailedToSerializeJSON(a), Self::FailedToSerializeJSON(b)) => a == b,
			_ => false,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::SerializingErr;
	use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;

	fn all() -> Vec<SerializingErr> {
		vec![
			SerializingErr::InvalidEndOfVarInt,
			SerializingErr::VarTypeTooLong("a".into()),
			SerializingErr::CouldNotDeserializeString(String::from_utf8(vec![0xFF]).unwrap_err()),
			SerializingErr::StringFromSliceError(<[u8; 2]>::try_from(&[0u8][..]).unwrap_err()),
			SerializingErr::InputEnded,
			SerializingErr::LeftoverInput,
			SerializingErr::UnknownFailure,
			SerializingErr::UniqueFailure("a".into()),
			SerializingErr::InvalidPacketState,
			SerializingErr::InvalidBitsPerEntry,
			SerializingErr::InvalidEnumValue(3),
			SerializingErr::MissingField("a".into()),
			SerializingErr::NoKnownPacket("a".into()),
			SerializingErr::DeserializationError("a".into()),
			SerializingErr::NbtError(NbtError::InvalidType),
			SerializingErr::InconsistentField("a".into()),
			SerializingErr::OutOfBounds("a".into()),
			SerializingErr::FailedToSerializeJSON("a".into()),
		]
	}

	/// Regression: seven variants fell through to `_ => false`, so `e == e.clone()` was false and
	/// `assert_eq!` on those errors could never pass. Every variant must equal itself and only itself.
	#[test]
	fn eq_is_reflexive_for_every_variant() {
		let errors = all();
		for (i, a) in errors.iter().enumerate() {
			for (j, b) in errors.iter().enumerate() {
				assert_eq!(a == b, i == j, "{a:?} vs {b:?}");
			}
		}
		assert_ne!(SerializingErr::InvalidEnumValue(1), SerializingErr::InvalidEnumValue(2));
	}

	/// Messages carry their detail text, and wrapped errors show the inner message unchanged.
	#[test]
	fn display_includes_detail() {
		assert_eq!(SerializingErr::OutOfBounds("x".into()).to_string(), "The deserialized data is not valid for the range of possible values: x");
		assert_eq!(SerializingErr::InvalidEnumValue(-4).to_string(), "The input data is not a valid enum value: -4");
		assert_eq!(SerializingErr::UniqueFailure("just this".into()).to_string(), "just this");
		assert_eq!(SerializingErr::NbtError(NbtError::InvalidType).to_string(), NbtError::InvalidType.to_string());
	}
}
