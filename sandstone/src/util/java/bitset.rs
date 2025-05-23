//! Implementation of https://docs.oracle.com/javase/8/docs/api/java/util/BitSet.html

use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use std::ops::Range;
use sandstone_derive::{McDeserialize, McSerialize};

/// A BitSet is a bitmask datatype of infinite size. It is stored as a Vec of u64
#[derive(McSerialize, McDeserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct BitSet {
	bits: Vec<u64>,
}

impl BitSet {
	/// Create a new bitset, defining the number of bits within the bitset
	pub fn new(size: usize) -> Self {
		let byte_size = (size + 63) / 64;
		Self {
			bits: vec![0; byte_size],
		}
	}

	pub fn get_bit(&self, index: usize) -> bool {
		let (byte_index, bit_index) = (index / 64, index % 64);
		if byte_index >= self.bits.len() {
			return false;
		}
		self.bits[byte_index] & (1 << bit_index) != 0
	}

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
	
	pub fn set_all(&mut self) {
		for byte in &mut self.bits {
			*byte = u64::MAX;
		}
	}
	
	pub fn flip(&mut self) {
		for byte in &mut self.bits {
			*byte = !*byte;
		}
	}
	
	pub fn slice(&self, range: Range<usize>) -> BitSet {
		let start = range.start / 64;
		let end = (range.end + 63) / 64;
		let mut bits = vec![0; end - start];
		for i in start..end {
			if i < self.bits.len() {
				bits[i - start] = self.bits[i];
			}
		}
		Self { bits }
	}

	pub fn clear_all(&mut self) {
		for byte in &mut self.bits {
			*byte = 0;
		}
	}

	pub fn clear(&mut self, index: usize) {
		let (byte_index, bit_index) = (index / 64, index % 64);
		if byte_index < self.bits.len() {
			self.bits[byte_index] &= !(1 << bit_index);
		}
	}

	pub fn size(&self) -> usize {
		self.bits.len() * 64
	}
	
	pub fn or(&mut self, other: &BitSet) {
		for (a, b) in self.bits.iter_mut().zip(other.bits.iter()) {
			*a |= *b;
		}
	}
	
	pub fn and(&mut self, other: &BitSet) {
		for (a, b) in self.bits.iter_mut().zip(other.bits.iter()) {
			*a &= *b;
		}
	}
	
	pub fn xor(&mut self, other: &BitSet) {
		for (a, b) in self.bits.iter_mut().zip(other.bits.iter()) {
			*a ^= *b;
		}
	}
	
	pub fn not(&mut self) {
		for byte in &mut self.bits {
			*byte = !*byte;
		}
	}
}

impl From<&[u8]> for BitSet {
	fn from(bytes: &[u8]) -> Self {
		let mut bits = vec![0; (bytes.len() + 7) / 8];
		for (i, byte) in bytes.iter().enumerate() {
			bits[i / 8] |= (*byte as u64) << (i % 8);
		}
		Self { bits }
	}
}

impl From<BitSet> for Vec<u8> {
	fn from(bitset: BitSet) -> Self {
		let mut bytes = vec![0; (bitset.size() + 7) / 8];
		for (i, byte) in bitset.bits.iter().enumerate() {
			bytes[i] = (*byte & 0xFF) as u8;
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
}