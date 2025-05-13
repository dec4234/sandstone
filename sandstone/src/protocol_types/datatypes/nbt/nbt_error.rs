use std::fmt::{Debug};

use thiserror::Error;

#[derive(Error, Debug, Clone, Hash, PartialEq, Eq)]
pub enum NbtError {
	#[error("Input ended prematurely")]
	InputEndedPrematurely,
	#[error("Unknown type number")]
	UnknownTypeNumber,
	#[error("Unexpected byte")]
	UnexpectedByte,
	#[error("Missing End Tag")]
	MissingEndTag,
	#[error("Mismatched types")]
	MismatchedTypes,
	#[error("End tag not allowed in list")]
	EndTagNotAllowedInList,
	#[error("Incompatible types")]
	IncompatibleTypes,
}