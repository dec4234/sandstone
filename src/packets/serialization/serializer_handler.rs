use std::cmp::min;
use std::fmt::{Debug, Display};

use anyhow::{anyhow, Result};

use crate::packets::packet_definer::PacketState;
use crate::packets::serialization::serializer_error::SerializingErr;

/// The result of a deserialization operation
pub type DeserializeResult<'a, T> = Result<T, SerializingErr>;

/// Handles the serialization of any types that `impl McSerialize`. Holds an
/// internal buffer representing the serialized data.
pub struct McSerializer {
	pub output: Vec<u8>
}

impl McSerializer {
	pub fn new() -> Self {
		Self {
			output: vec![]
		}
	}

	/// Clear the existing serialized data from the internal buffer
	pub fn clear(&mut self) {
		self.output.clear();
	}

	/// Add a slice of bytes to the internal buffer. Reallocates the buffer
	/// for the new amount of space required before pushing.
	pub fn serialize_bytes(&mut self, input: &[u8]) {
		let mut i = self.output.len();
		self.output.resize(self.output.len() + input.len(), 1); // maybe this is helpful?

		for b in input {
			self.output[i] = *b;
			i += 1;
		}
	}

	pub fn serialize_vec(&mut self, vec: Vec<u8>) {
		self.serialize_bytes(vec.as_slice());
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

	pub fn pop(&mut self) -> Option<u8> {
		if self.index < self.data.len() {
			let u = self.data[self.index];
			self.increment(1);
			Some(u)
		} else {
			None
		}
	}

	pub fn increment(&mut self, amount: usize) {
		self.index += amount;
	}

	pub fn increment_by_diff(&mut self, other: usize) {
		if other > self.index {
			self.increment(other - self.index);
		}
	}

	pub fn isAtEnd(&self) -> bool {
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
	
	pub fn sub_deserializer_length(&mut self, end: usize) -> Result<McDeserializer> {
		if self.index + end > self.data.len() {
			return Err(anyhow!("Sub-deserializer length exceeds data length"));
		}
		
		let ret = Ok(McDeserializer::new(&self.data[self.index..(self.index + end)]));
		
		self.index += end;
		
		ret
	}
}

/// The standard deserializer used for most regular deserialization operations. Converts
/// byte data into rust structs and primitive data types
pub trait McDeserialize {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> where Self: Sized;
}

/// Deserialize data given the current packet state and the packet id. This is needed since
/// the packet id is not enough to determine the packet type in some cases. 
/// (ie. Both STATUS and HANDSHAKING states have a packet with ID 0)
pub trait StateBasedDeserializer {
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: &PacketState) -> DeserializeResult<'a, Self> where Self: Sized;
}

/// Serialize a struct into a byte buffer to be sent over TCP
pub trait McSerialize {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr>;
}