//! Implementation of https://docs.oracle.com/javase/8/docs/api/java/util/BitSet.html

use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use std::ops::Range;

/// A BitSet is a bitmask datatype of infinite size. It is stored as a Vec of u64
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Data_types#BitSet>
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BitSet {
	bits: Vec<u64>,
}

impl McSerialize for BitSet {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		VarInt(self.bits.len() as i32).mc_serialize(serializer)?;
		for long in &self.bits {
			long.mc_serialize(serializer)?;
		}
		Ok(())
	}
}

impl McDeserialize for BitSet {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let len = VarInt::mc_deserialize(deserializer)?;
		let mut bits = Vec::with_capacity(deserializer.prealloc_capacity(len.0)?);
		for _ in 0..len.0 {
			bits.push(u64::mc_deserialize(deserializer)?);
		}
		Ok(Self {
			bits,
		})
	}
}

impl BitSet {
	/// Create a new bitset, defining the number of bits within the bitset
	pub fn new(size: usize) -> Self {
		let byte_size = size.div_ceil(64);
		Self {
			bits: vec![0; byte_size],
		}
	}

	/// Get the value of a specific bit in the BitSet
	pub fn get_bit(&self, index: usize) -> bool {
		let (byte_index, bit_index) = (index / 64, index % 64);
		if byte_index >= self.bits.len() {
			return false;
		}
		self.bits[byte_index] & (1 << bit_index) != 0
	}

	/// Set a specific bit in the BitSet to true (1) or false (0)
	pub fn set_bit(&mut self, index: usize, value: bool) {
		let (byte_index, bit_index) = (index / 64, index % 64);
		if byte_index >= self.bits.len() {
			return;
		}
		if value {
			self.bits[byte_index] |= 1 << bit_index;
		} else {
			self.bits[byte_index] &= !(1 << bit_index);
		}
	}

	/// Set the first u64 value in the BitSet to the given value.
	pub fn add_val(&mut self, value: u64) {
		if self.bits.is_empty() {
			self.bits.push(value);
		} else {
			self.bits[0] = value;
		}
	}

	/// Set all bits in the BitSet to true (1)
	pub fn set_all(&mut self) {
		for byte in &mut self.bits {
			*byte = u64::MAX;
		}
	}

	/// Flip all bits in the BitSet (0 becomes 1, 1 becomes 0)
	pub fn flip(&mut self) {
		for byte in &mut self.bits {
			*byte = !*byte;
		}
	}

	/// Create a new BitSet from the bits in `range`, like Java's `BitSet.get(from, to)`.
	/// Bit `range.start` becomes bit 0 of the result. Bits outside the range are not copied.
	pub fn slice(&self, range: Range<usize>) -> BitSet {
		let mut slice = BitSet::new(range.len());
		for (i, index) in range.enumerate() {
			slice.set_bit(i, self.get_bit(index));
		}
		slice
	}

	/// Clear all bits in the BitSet (set all bits to false)
	pub fn clear_all(&mut self) {
		for byte in &mut self.bits {
			*byte = 0;
		}
	}

	/// Clear a specific bit in the BitSet (set it to false)
	pub fn clear(&mut self, index: usize) {
		let (byte_index, bit_index) = (index / 64, index % 64);
		if byte_index < self.bits.len() {
			self.bits[byte_index] &= !(1 << bit_index);
		}
	}

	/// Get the number of bits in the BitSet
	pub fn size(&self) -> usize {
		self.bits.len() * 64
	}

	/// Or the bits of this BitSet with another BitSet, modifying this BitSet in place
	pub fn or(&mut self, other: &BitSet) {
		for (a, b) in self.bits.iter_mut().zip(other.bits.iter()) {
			*a |= *b;
		}
	}

	/// And the bits of this BitSet with another BitSet, modifying this BitSet in place
	pub fn and(&mut self, other: &BitSet) {
		for (a, b) in self.bits.iter_mut().zip(other.bits.iter()) {
			*a &= *b;
		}
	}

	/// Xor the bits of this BitSet with another BitSet, modifying this BitSet in place
	pub fn xor(&mut self, other: &BitSet) {
		for (a, b) in self.bits.iter_mut().zip(other.bits.iter()) {
			*a ^= *b;
		}
	}

	/// Negate the bits of this BitSet, modifying this BitSet in place
	pub fn not(&mut self) {
		self.flip();
	}
}

impl McDefault for BitSet {
	fn mc_default() -> Self {
		let mut bit = Self::new(6);

		bit.add_val(u64::mc_default());

		bit
	}
}

impl From<&[u8]> for BitSet {
	fn from(bytes: &[u8]) -> Self {
		let mut bits = vec![0; bytes.len().div_ceil(8)];
		for (i, byte) in bytes.iter().enumerate() {
			bits[i / 8] |= (*byte as u64) << ((i % 8) * 8);
		}
		Self {
			bits,
		}
	}
}

impl From<BitSet> for Vec<u8> {
	fn from(bitset: BitSet) -> Self {
		// Same as Java's `BitSet.toByteArray`: little-endian, with trailing zero bytes removed.
		let mut bytes: Vec<u8> = bitset.bits.iter().flat_map(|long| long.to_le_bytes()).collect();
		while bytes.last() == Some(&0) {
			bytes.pop();
		}
		bytes
	}
}

#[cfg(test)]
mod test {
	use crate::util::java::bitset::BitSet;

	#[test]
	fn test_bitset() {
		let mut bitset = BitSet::new(128);

		bitset.set_bit(0, true);
		bitset.set_bit(63, true);
		bitset.set_bit(64, true);
		bitset.set_bit(127, true);

		assert!(bitset.get_bit(0));
		assert!(bitset.get_bit(63));
		assert!(bitset.get_bit(64));
		assert!(bitset.get_bit(127));

		assert!(!bitset.get_bit(1));
		assert!(!bitset.get_bit(62));
		assert!(!bitset.get_bit(65));
		assert!(!bitset.get_bit(126));
	}

	/// On the wire a BitSet is a VarInt count of longs, then each long big-endian. Light masks in
	/// chunk packets use this, so a wrong layout puts light data in the wrong sections.
	#[test]
	fn wire_format_round_trip() {
		use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
		let mut bitset = BitSet::new(128);
		bitset.set_bit(0, true);
		bitset.set_bit(127, true);

		let mut serializer = McSerializer::new();
		bitset.mc_serialize(&mut serializer).unwrap();
		assert_eq!(serializer.output, vec![2, 0, 0, 0, 0, 0, 0, 0, 1, 0x80, 0, 0, 0, 0, 0, 0, 0]);

		let mut de = McDeserializer::new(&serializer.output);
		assert_eq!(BitSet::mc_deserialize(&mut de).unwrap(), bitset);
		assert!(de.is_at_end());
	}

	/// Regression: the count comes from the peer and went straight to Vec::with_capacity. A
	/// negative count panicked and a very large count aborted the process.
	#[test]
	fn bad_count_errors() {
		use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
		use crate::protocol_types::datatypes::var_types::VarInt;
		for count in [-1, i32::MAX] {
			let mut serializer = McSerializer::new();
			VarInt(count).mc_serialize(&mut serializer).unwrap();
			assert!(BitSet::mc_deserialize(&mut McDeserializer::new(&serializer.output)).is_err(), "count {count}");
		}
	}

	/// The size is fixed at creation. Writes past it are ignored, and reads past it are false, so
	/// the set never grows by surprise.
	#[test]
	fn out_of_range_and_clear() {
		let mut bitset = BitSet::new(64);
		assert_eq!(bitset.size(), 64);
		bitset.set_bit(64, true);
		assert!(!bitset.get_bit(64));
		assert_eq!(bitset.size(), 64);

		bitset.set_bit(5, true);
		bitset.set_bit(5, false);
		assert!(!bitset.get_bit(5));

		bitset.set_all();
		assert!((0..64).all(|i| bitset.get_bit(i)));
		bitset.clear(3);
		assert!(!bitset.get_bit(3));
		bitset.clear_all();
		assert!((0..64).all(|i| !bitset.get_bit(i)));
	}

	#[test]
	fn flip_test() {
		let mut bitset = BitSet::new(128);
		bitset.set_bit(0, true);
		bitset.set_bit(63, true);
		bitset.set_bit(64, true);
		bitset.set_bit(127, true);

		assert!(bitset.get_bit(0));
		assert!(bitset.get_bit(63));
		assert!(bitset.get_bit(64));
		assert!(bitset.get_bit(127));

		bitset.flip();

		assert!(!bitset.get_bit(0));
		assert!(!bitset.get_bit(63));
		assert!(!bitset.get_bit(64));
		assert!(!bitset.get_bit(127));
	}

	#[test]
	fn slicing() {
		let mut bitset = BitSet::new(128);
		bitset.set_bit(0, true);
		bitset.set_bit(63, true);
		bitset.set_bit(64, true);
		bitset.set_bit(127, true);

		let slice = bitset.slice(0..64);
		assert!(slice.get_bit(0));
		assert!(slice.get_bit(63));
		assert!(!slice.get_bit(64));
		assert!(!slice.get_bit(127));

		let slice = bitset.slice(64..128);
		assert!(slice.get_bit(0));
		assert!(!slice.get_bit(62));
		assert!(slice.get_bit(63));
	}

	/// Regression: slice copied whole u64 words, so a start that was not a multiple of 64 was
	/// ignored and bits at or after the end were kept. Bit `start` must become bit 0 and only
	/// the bits in the range may be copied, the same as Java's `BitSet.get(from, to)`.
	#[test]
	fn slicing_off_word_boundary() {
		let mut bitset = BitSet::new(128);
		bitset.set_bit(9, true);
		bitset.set_bit(10, true);
		bitset.set_bit(70, true);
		bitset.set_bit(75, true);

		let slice = bitset.slice(10..75);
		assert_eq!(slice.size(), 128);
		assert!(slice.get_bit(0), "bit 10 must move to bit 0");
		assert!(slice.get_bit(60), "bit 70 must move to bit 60 across the word boundary");
		assert!(!slice.get_bit(65), "bit 75 is at the end of the range and must not be copied");
		let set: Vec<usize> = (0..slice.size()).filter(|&i| slice.get_bit(i)).collect();
		assert_eq!(set, vec![0, 60], "bit 9 is before the range and must not be copied");

		// Bits past the end of the source read as false.
		let slice = bitset.slice(120..200);
		assert!((0..slice.size()).all(|i| !slice.get_bit(i)));
	}

	#[test]
	fn test_or_and() {
		let mut bitset1 = BitSet::new(128);
		let mut bitset2 = BitSet::new(128);

		bitset1.set_bit(0, true);
		bitset1.set_bit(63, true);
		bitset2.set_bit(0, true);
		bitset2.set_bit(63, true);

		bitset1.or(&bitset2);

		assert!(bitset1.get_bit(0));
		assert!(bitset1.get_bit(63));
		assert!(!bitset1.get_bit(61));
		assert!(!bitset1.get_bit(127));

		bitset2.set_bit(60, true);

		bitset1.and(&bitset2);

		assert!(bitset1.get_bit(0));
		assert!(bitset1.get_bit(63));
		assert!(!bitset1.get_bit(61));
		assert!(!bitset1.get_bit(127));
		assert!(!bitset1.get_bit(60));
	}

	#[test]
	fn test_xor_not() {
		let mut bitset1 = BitSet::new(128);
		let mut bitset2 = BitSet::new(128);

		bitset1.set_bit(0, true);
		bitset1.set_bit(63, true);
		bitset2.set_bit(0, true);
		bitset2.set_bit(63, true);

		bitset1.xor(&bitset2);

		assert!(!bitset1.get_bit(0));
		assert!(!bitset1.get_bit(63));
		assert!(!bitset1.get_bit(61));
		assert!(!bitset1.get_bit(127));

		bitset1.not();

		assert!(bitset1.get_bit(0));
		assert!(bitset1.get_bit(63));
		assert!(bitset1.get_bit(61));
		assert!(bitset1.get_bit(127));
	}

	/// Byte `i` holds bits `8*i .. 8*i+8`, the same little-endian layout as Java's `BitSet.valueOf(byte[])`.
	/// If the shift is wrong, a bit set from bytes reads back at the wrong index.
	#[test]
	fn from_bytes_places_each_byte_at_its_own_offset() {
		let bitset = BitSet::from(&[0x01u8, 0x01, 0, 0, 0, 0, 0, 0x80, 0x02][..]);
		assert!(bitset.get_bit(0));
		assert!(!bitset.get_bit(1));
		assert!(bitset.get_bit(8));
		assert!(bitset.get_bit(63));
		assert!(bitset.get_bit(65));
		assert!(!bitset.get_bit(64));
	}

	/// Converting to bytes must keep all 8 bytes of each u64, or high bits are lost.
	#[test]
	fn to_bytes_keeps_every_byte() {
		let mut bitset = BitSet::new(128);
		bitset.set_bit(0, true);
		bitset.set_bit(15, true);
		bitset.set_bit(63, true);
		bitset.set_bit(64, true);
		bitset.set_bit(127, true);

		let bytes: Vec<u8> = bitset.clone().into();
		assert_eq!(bytes.len(), 16);
		assert_eq!(bytes[0], 0x01);
		assert_eq!(bytes[1], 0x80);
		assert_eq!(bytes[7], 0x80);
		assert_eq!(bytes[8], 0x01);
		assert_eq!(bytes[15], 0x80);
		assert_eq!(BitSet::from(bytes.as_slice()), bitset);
	}

	/// Java's `toByteArray` gives `(length() + 7) / 8` bytes, where `length()` is the highest set bit + 1.
	/// Trailing zero bytes are removed, so a peer that compares byte arrays gets the same result.
	#[test]
	fn to_bytes_removes_trailing_zero_bytes_like_java() {
		let mut bitset = BitSet::new(128);
		assert_eq!(Vec::<u8>::from(bitset.clone()), Vec::<u8>::new());

		bitset.set_bit(9, true);
		assert_eq!(Vec::<u8>::from(bitset.clone()), vec![0x00, 0x02]);

		bitset.set_bit(64, true);
		assert_eq!(Vec::<u8>::from(bitset), vec![0x00, 0x02, 0, 0, 0, 0, 0, 0, 0x01]);
	}
}
