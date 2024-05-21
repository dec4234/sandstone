use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Pointer, Write};

#[derive(Clone, PartialEq, Eq)]
pub struct NoDataReceivedError;

impl Debug for NoDataReceivedError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("No data received")
	}
}

impl Display for NoDataReceivedError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("No data received")
	}
}

impl Error for NoDataReceivedError {
	
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConnectionAbortedLocally;

impl Debug for ConnectionAbortedLocally {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("The connection was aborted by the local machine")
	}
}

impl Display for ConnectionAbortedLocally {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("The connection was aborted by the local machine")
	}
}

impl Error for ConnectionAbortedLocally {
	
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConnectionAbortedRemotely;

impl Debug for ConnectionAbortedRemotely {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("The connection was aborted by the remote machine")
	}
}

impl Display for ConnectionAbortedRemotely {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("The connection was aborted by the remote machine")
	}
}

impl Error for ConnectionAbortedRemotely {
	
}

#[derive(Clone, PartialEq, Eq)]
pub struct InvalidPacketState;

impl Debug for InvalidPacketState {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("Invalid packet state")
	}
}

impl Display for InvalidPacketState {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_str("Invalid packet state")
	}
}

impl Error for InvalidPacketState {
	
}