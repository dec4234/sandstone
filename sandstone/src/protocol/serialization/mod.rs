//! This file defines the handlers for serialization and deserialization in the API.
//! Information can be "serialized" from the original type into its raw bytes. This can then be
//! sent in a packet over the network.
//! Conversely, information can also be "deserialized" from raw bytes into the original type. This is useful
//! for reading packets from the network.

use std::cmp::min;

use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::serialization::serializer_error::SerializingErr;

mod serializer_types;
pub mod serializer_error;
mod serializer_testing;

/// The result of a serialization/deserialization operation.
/// See [SerializingErr] for more information on the error types
pub type SerializingResult<'a, T> = Result<T, SerializingErr>;

/// Handles the serialization of any types that `impl McSerialize`. Holds an
/// internal buffer representing the serialized data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct McSerializer {
	pub output: Vec<u8>
}

impl McSerializer {
	pub fn new() -> Self {
		Self {
			output: vec![]
		}
	}
	
	/// Initialize the size of the internal serializer buffer. If you plan on serializing a lot of small
	/// items, then this should be used to avoid unnecessary reallocations.
	pub fn init_size(size: usize) -> Self {
		Self {
			output: Vec::with_capacity(size)
		}
	}
	
	/// Set the size of the internal buffer. This will resize the buffer to the size specified.
	/// The provided size must be greater than the current length of the buffer.
	pub fn set_size(&mut self, size: usize) -> SerializingResult<()> {
		if size < self.output.len() {
			return Err(SerializingErr::UniqueFailure("Cannot set size to less than current length".to_string()));
		}
		
		self.output.reserve(size);
		
		Ok(())
	}

	/// Clear the existing serialized data from the internal buffer
	pub fn clear(&mut self) {
		self.output.clear();
	}

	/// Add a slice of bytes to the internal buffer. Reallocates the buffer
	/// for the new amount of space required before pushing.
	pub fn serialize_bytes(&mut self, input: &[u8]) {
		for b in input {
			self.output.push(*b);
		}
	}

	pub fn serialize_vec(&mut self, vec: Vec<u8>) {
		self.serialize_bytes(&vec);
	}

	pub fn serialize_u8(&mut self, b: u8) {
		self.output.push(b);
	}

	/// Serialized as is, NO LENGTH PREFIX
	pub fn serialize_str_no_length_prefix(&mut self, s: &str) {
		self.serialize_bytes(s.as_bytes());
	}

	pub fn get_last(&self) -> Option<&u8> {
		self.output.last()
	}

	pub fn merge(&mut self, serializer: McSerializer) {
		self.serialize_bytes(&serializer.output);
	}
}

/// Helper for deserializing byte data into types that `impl McDeserialize`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct McDeserializer<'a> {
	pub data: &'a [u8],
	pub index: usize
}

impl <'a> McDeserializer<'a> {
	pub fn new(data: &'a [u8]) -> Self {
		Self {
			data,
			index: 0
		}
	}

	/// Collect the remaining data into a sub-slice
	pub fn collect_remaining(&self) -> &[u8] {
		&self.data[self.index..]
	}

	/// Slice the internal buffer, starting at the current index and up to the bound provided. Will
	/// cut off the subslice at max(data.len, bound + index) to prevent overflow
	pub fn slice(&mut self, bound: usize) -> &[u8] {
		let actual = min(self.data.len(), bound) + self.index;
		let actual = min(actual, self.data.len());

		let slice = &self.data[self.index..actual];
		self.increment(slice.len());
		slice
	}

	/// Slice the internal buffer, starting at the current index and up to the
	/// bound provided, but only if it is within bounds
	pub fn slice_option(&mut self, bound: usize) -> Option<&[u8]> {
		if self.index + bound > self.data.len() {
			return None;
		}

		let slice = &self.data[self.index..(self.index + bound)];
		self.increment(bound);
		Some(slice)
	}

	/// Pop a single byte from the buffer and return it if the buffer is not empty.
	pub fn pop(&mut self) -> Option<u8> {
		if self.index < self.data.len() {
			let u = self.data[self.index];
			self.increment(1);
			Some(u)
		} else {
			None
		}
	}

	/// Increment the index of this McDeserializer by the amount specified
	pub fn increment(&mut self, amount: usize) {
		self.index += amount;
	}

	/// Increment the index of this McDeserializer by the difference between the current index
	/// and the provided index.
	pub fn increment_by_diff(&mut self, other: usize) {
		if other > self.index {
			self.increment(other - self.index);
		}
	}

	pub fn is_at_end(&self) -> bool {
		self.index >= self.data.len()
	}

	pub fn reset(&mut self) {
		self.index = 0;
	}

	/// Creates a new McDeserializer only including the remaining unused data.
	/// Used in conjunction with reset()
	pub fn create_sub_deserializer(&self) -> McDeserializer {
		McDeserializer::new(&self.data[self.index..])
	}

	/// Create a new McDeserializer with a start at `index` and an end at `index + end`.
	/// Basically reserves the number of bytes you specify for the sub-deserializer.
	/// Also increments the parent McDeserializer's index by `end`
	pub fn sub_deserializer_length(&mut self, end: usize) -> SerializingResult<McDeserializer> {
		if self.index + end > self.data.len() {
			return Err(SerializingErr::UniqueFailure("Sub-deserializer length exceeds data length".to_string()));
		}

		let ret = Ok(McDeserializer::new(&self.data[self.index..(self.index + end)]));

		self.index += end;

		ret
	}
}

/// The standard deserializer used for most regular deserialization operations. Converts
/// byte data into rust structs and primitive data types
pub trait McDeserialize {
	/// Deserialize the byte buffer into the type that implements this trait.
	/// Note that if the byte buffer does not match the type you are trying to deserialize, or it does not
	/// contain enough data then this will return an error.
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized;
}

/// Deserialize data given the current packet state and the packet id. This is needed since
/// the packet id is not enough to determine the packet type in some cases.
/// (ie. Both STATUS and HANDSHAKING states have a packet with ID 0)
pub trait StateBasedDeserializer {
	/// Deserialize the byte buffer into a 'Packet'. This takes 2 extra arguments, the packet state and the 
	/// direction of the packet to narrow down the exact packet that should be deserialized.
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: PacketState, packet_direction: PacketDirection) -> SerializingResult<'a, Self> where Self: Sized;
}

/// Implement this on a type to enable serializing it into a byte buffer. All types that will be sent
/// or received from the network must implement this type.
pub trait McSerialize {
	/// Serialize the type that implements this trait into a byte buffer. This enables the struct
	/// to be sent over the Minecraft network.
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()>;
}
