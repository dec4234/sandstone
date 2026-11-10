//! This file defines the handlers for serialization and deserialization in the API.
//! Information can be "serialized" from the original type into its raw bytes. This can then be
//! sent in a packet over the network.
//! Conversely, information can also be "deserialized" from raw bytes into the original type. This is useful
//! for reading packets from the network.

use std::cmp::min;

use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::serialization::serializer_error::SerializingErr;

pub mod serializer_error;
mod serializer_testing;
pub mod serializer_types;

/// The result of a serialization/deserialization operation.
/// See [SerializingErr] for more information on the error types
pub type SerializingResult<'a, T> = Result<T, SerializingErr>;

/// Handles the serialization of any types that `impl McSerialize`. Holds an
/// internal buffer representing the serialized data.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct McSerializer {
	pub output: Vec<u8>,
}

impl McSerializer {
	pub fn new() -> Self {
		Self {
			output: vec![],
		}
	}

	/// Initialize the size of the internal serializer buffer. If you plan on serializing a lot of small
	/// items, then this should be used to avoid unnecessary reallocations.
	pub fn init_size(size: usize) -> Self {
		Self {
			output: Vec::with_capacity(size),
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

	/// Serialize a Vec of bytes and fix it to the end of the internal buffer.
	pub fn serialize_vec(&mut self, vec: Vec<u8>) {
		self.serialize_bytes(&vec);
	}

	/// Add a single byte to the end of the internal buffer.
	pub fn serialize_u8(&mut self, b: u8) {
		self.output.push(b);
	}

	/// Serialized as is, NO LENGTH PREFIX
	pub fn serialize_str_no_length_prefix(&mut self, s: &str) {
		self.serialize_bytes(s.as_bytes());
	}

	/// Return the last byte in the internal buffer, if it exists.
	pub fn get_last(&self) -> Option<&u8> {
		self.output.last()
	}

	/// Merge the contents of the provided McSerializer into this one. The contents of the provided serializer
	/// are appended to the end of the current serializer's output.
	pub fn merge(&mut self, serializer: McSerializer) {
		self.serialize_bytes(&serializer.output);
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.output
	}
}

impl From<&[u8]> for McSerializer {
	/// Create a new McSerializer from a byte slice. This will copy the bytes into the internal buffer.
	fn from(input: &[u8]) -> Self {
		let mut serializer = McSerializer::new();
		serializer.serialize_bytes(input);
		serializer
	}
}

impl From<McSerializer> for Vec<u8> {
	/// Convert a McSerializer into a Vec<u8>. This is useful for sending the serialized data over the network.
	fn from(serializer: McSerializer) -> Self {
		serializer.output
	}
}

/// Helper for deserializing byte data into types that `impl McDeserialize`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct McDeserializer<'a> {
	pub data: &'a [u8],
	pub index: usize,
}

impl<'a> McDeserializer<'a> {
	/// Create a new McDeserializer from a slice of bytes.
	pub fn new(data: &'a [u8]) -> Self {
		Self {
			data,
			index: 0,
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

	/// Peek at the next byte in the buffer without modifying the index or consuming the byte.
	pub fn peek_byte(&self) -> Option<u8> {
		if self.index < self.data.len() { Some(self.data[self.index]) } else { None }
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

	/// Returns a safe initial capacity for a list of `count` items read from a peer.
	///
	/// A negative count is an error. The result is capped at the number of bytes that remain,
	/// because a peer can send a very large count, and `Vec::with_capacity` then aborts the
	/// process. The list can still grow past this capacity if its items use less than one byte.
	pub fn prealloc_capacity<'b>(&self, count: i32) -> SerializingResult<'b, usize> {
		if count < 0 {
			return Err(SerializingErr::OutOfBounds(format!("Length prefix of {count} is negative")));
		}
		Ok(min(count as usize, self.data.len().saturating_sub(self.index)))
	}

	/// Return true if the current index is at the end of the data buffer.
	pub fn is_at_end(&self) -> bool {
		self.index >= self.data.len()
	}

	/// Set the internal index back to the start of the data buffer.
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

	/// Create a sub-slice of the data, before and after the current index. Useful for debugging.
	/// Set start rel to the number of bytes before the current index, and end rel to the number of bytes after the current index.
	pub fn subset(&self, mut start_rel: usize, mut end_rel: usize) -> &[u8] {
		start_rel = self.index.saturating_sub(start_rel);
		end_rel = self.index.saturating_add(end_rel);

		&self.data[start_rel..end_rel]
	}
}

/// The standard deserializer used for most regular deserialization operations. Converts
/// byte data into rust structs and primitive data types
pub trait McDeserialize {
	/// Deserialize the byte buffer into the type that implements this trait.
	/// Note that if the byte buffer does not match the type you are trying to deserialize, or it does not
	/// contain enough data then this will return an error.
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized;
}

/// Deserialize data given the current packet state and the packet id. This is needed since
/// the packet id is not enough to determine the packet type in some cases.
/// (ie. Both STATUS and HANDSHAKING states have a packet with ID 0)
pub trait StateBasedDeserializer {
	/// Deserialize the byte buffer into a 'Packet'. This takes 2 extra arguments, the packet state and the
	/// direction of the packet to narrow down the exact packet that should be deserialized.
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: PacketState, packet_direction: PacketDirection) -> SerializingResult<'a, Self>
	where
		Self: Sized;
}

/// Implement this on a type to enable serializing it into a byte buffer. All types that will be sent
/// or received from the network must implement this type.
pub trait McSerialize {
	/// Serialize the type that implements this trait into a byte buffer. This enables the struct
	/// to be sent over the Minecraft network.
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()>;
}

#[cfg(test)]
mod tests {
	use crate::protocol::serialization::serializer_error::SerializingErr;
	use crate::protocol::serialization::{McDeserializer, McSerializer};

	/// set_size must refuse to shrink below already-written data, because that would imply
	/// discarding bytes the caller has already serialized.
	#[test]
	fn set_size_rejects_shrink_and_reserves_growth() {
		let mut serializer = McSerializer::from(&[1u8, 2, 3][..]);
		assert!(matches!(serializer.set_size(2), Err(SerializingErr::UniqueFailure(_))));
		assert_eq!(serializer.as_bytes(), &[1, 2, 3], "a failed set_size must not touch the data");

		serializer.set_size(64).unwrap();
		assert!(serializer.output.capacity() >= 64);
		assert_eq!(serializer.as_bytes(), &[1, 2, 3], "set_size must not change the data");
	}

	#[test]
	fn init_size_preallocates_without_data() {
		let serializer = McSerializer::init_size(32);
		assert!(serializer.output.capacity() >= 32);
		assert!(serializer.output.is_empty());
	}

	/// Packet framing relies on merge appending after existing bytes (length + id first, then body).
	#[test]
	fn merge_appends_in_order() {
		let mut head = McSerializer::new();
		head.serialize_u8(0xAA);
		let mut body = McSerializer::new();
		body.serialize_vec(vec![1, 2]);
		body.serialize_bytes(&[3]);

		head.merge(body);
		assert_eq!(head.as_bytes(), &[0xAA, 1, 2, 3]);
		assert_eq!(head.get_last(), Some(&3));
	}

	/// Raw strings are used where the protocol has no length prefix, so no VarInt may be written.
	#[test]
	fn str_without_length_prefix_is_raw_utf8() {
		let mut serializer = McSerializer::new();
		serializer.serialize_str_no_length_prefix("hi");
		assert_eq!(serializer.as_bytes(), b"hi");
	}

	#[test]
	fn clear_and_conversions() {
		let mut serializer = McSerializer::from(&[9u8, 8][..]);
		let bytes: Vec<u8> = serializer.clone().into();
		assert_eq!(bytes, vec![9, 8]);

		serializer.clear();
		assert!(serializer.as_bytes().is_empty());
		assert_eq!(serializer.get_last(), None);
	}

	#[test]
	fn pop_and_peek() {
		let data = [5u8, 6];
		let mut de = McDeserializer::new(&data);
		assert_eq!(de.peek_byte(), Some(5));
		assert_eq!(de.index, 0, "peek must not consume");
		assert_eq!(de.pop(), Some(5));
		assert_eq!(de.pop(), Some(6));
		assert!(de.is_at_end());
		assert_eq!(de.pop(), None);
		assert_eq!(de.peek_byte(), None);
	}

	/// slice is lenient: it clamps to the end of the data instead of panicking.
	#[test]
	fn slice_clamps_to_end_of_data() {
		let data = [1u8, 2, 3, 4];
		let mut de = McDeserializer::new(&data);
		de.increment(1);
		assert_eq!(de.slice(2), &[2, 3]);
		assert_eq!(de.index, 3);
		assert_eq!(de.slice(100), &[4]);
		assert!(de.is_at_end());
		assert_eq!(de.slice(1), &[] as &[u8]);
	}

	/// slice_option is strict: an out-of-range request returns None and must not move the index,
	/// so a caller can report an error without corrupting the read position.
	#[test]
	fn slice_option_is_strict() {
		let data = [1u8, 2, 3];
		let mut de = McDeserializer::new(&data);
		assert_eq!(de.slice_option(4), None);
		assert_eq!(de.index, 0);
		assert_eq!(de.slice_option(3), Some(&[1u8, 2, 3][..]));
		assert!(de.is_at_end());
	}

	#[test]
	fn increment_by_diff_only_moves_forward() {
		let data = [0u8; 8];
		let mut de = McDeserializer::new(&data);
		de.increment(4);
		de.increment_by_diff(2);
		assert_eq!(de.index, 4, "a smaller target index must not move the deserializer back");
		de.increment_by_diff(6);
		assert_eq!(de.index, 6);
	}

	#[test]
	fn reset_and_remaining() {
		let data = [1u8, 2, 3];
		let mut de = McDeserializer::new(&data);
		de.increment(2);
		assert_eq!(de.collect_remaining(), &[3]);

		let sub = de.create_sub_deserializer();
		assert_eq!(sub.data, &[3]);
		assert_eq!(sub.index, 0);
		assert_eq!(de.index, 2, "creating a sub-deserializer must not consume the parent");

		de.reset();
		assert_eq!(de.index, 0);
		assert_eq!(de.collect_remaining(), &data);
	}

	/// Packet deserialization reserves exactly `length` bytes for the packet body. The parent must skip
	/// those bytes so the next packet in the stream starts at the right place.
	#[test]
	fn sub_deserializer_length_reserves_bytes() {
		let data = [1u8, 2, 3, 4, 5];
		let mut de = McDeserializer::new(&data);
		de.increment(1);
		{
			let sub = de.sub_deserializer_length(3).unwrap();
			assert_eq!(sub.data, &[2, 3, 4]);
			assert_eq!(sub.index, 0);
		}
		assert_eq!(de.index, 4);

		assert!(matches!(de.sub_deserializer_length(2), Err(SerializingErr::UniqueFailure(_))));
		assert_eq!(de.index, 4, "a failed reservation must not move the parent");
	}

	#[test]
	fn subset_around_index() {
		let data = [0u8, 1, 2, 3, 4, 5];
		let mut de = McDeserializer::new(&data);
		de.increment(3);
		assert_eq!(de.subset(2, 2), &[1, 2, 3, 4]);
		assert_eq!(de.subset(10, 1), &[0, 1, 2, 3], "start must saturate at 0");
	}
}
