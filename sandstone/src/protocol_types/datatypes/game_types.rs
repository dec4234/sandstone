//! Types found in game such as position, etc.

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

/// A Minecraft position, internally represented as a 64-bit integer.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct Position {
	data: u64
}

impl Position {
	pub fn new(x: i64, y: i64, z: i64) -> Self {
		let data: u64 = (((x & 0x3FFFFFF) << 38) | ((z & 0x3FFFFFF) << 12) | (y & 0xFFF)) as u64;
		Self {
			data
		}
	}

	fn sign_extend(value: u64, bits: u32) -> i64 {
		let shift = 64 - bits;
		((value << shift) as i64) >> shift
	}

	pub fn x(&self) -> i64 {
		let raw = (self.data >> 38) & 0x3FFFFFF;
		Self::sign_extend(raw, 26)
	}

	pub fn y(&self) -> i64 {
		let raw = self.data & 0xFFF;
		Self::sign_extend(raw, 12)
	}

	pub fn z(&self) -> i64 {
		let raw = (self.data >> 12) & 0x3FFFFFF;
		Self::sign_extend(raw, 26)
	}
}

/// Represents a packed i64 (long) that contains block or biome data. See 
/// https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Data_Array_format for more info. This
/// matches the spec for packed data after 1.16
#[derive(McDefault, Debug, Clone, Hash, PartialEq)]
pub struct PackedEntries {
	data: i64,
	/// The number of bits allocated to each entry
	bpe: u8,
}

impl PackedEntries {
	pub fn new(bpe: u8) -> Self {
		Self {
			data: 0,
			bpe,
		}
	}

	/// Get the entry by the index from the packed i64. The first entry occupies the least significant bits
	pub fn get_entry(&self, index: u8) -> u64 {
		let mask = (1 << self.bpe) - 1;
		let shift = index * self.bpe;
		((self.data >> shift) & mask as i64) as u64
	}

	pub fn set_entry(&mut self, index: u8, value: u64) {
		let mask = (1 << self.bpe) - 1;
		let shift = index * self.bpe;
		self.data &= !(mask << shift);
		self.data |= ((value & mask as u64) << shift ) as i64;
	}

	/// A nonstandard deserializer that utilizes bits per entry
	pub(crate) fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer, bpe: u8) -> SerializingResult<'a, Self> where Self: Sized {
		let data = i64::mc_deserialize(deserializer)?;

		Ok(Self {
			data,
			bpe
		})
	}
}

impl McSerialize for PackedEntries {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.data.mc_serialize(serializer)?;
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use crate::protocol_types::datatypes::game_types::{PackedEntries, Position};

	#[test]
	fn test_position() {
		let pos = Position::new(1, 2, 3);
		assert_eq!(pos.x(), 1);
		assert_eq!(pos.y(), 2);
		assert_eq!(pos.z(), 3);

		let pos = Position::new(-1, -2, -3);
		assert_eq!(pos.x(), -1);
		assert_eq!(pos.y(), -2);
		assert_eq!(pos.z(), -3);
	}

	#[test]
	fn basic_packed_entries_test() {
		let mut packed = super::PackedEntries::new(4);
		packed.set_entry(0, 1);
		packed.set_entry(1, 2);
		packed.set_entry(2, 3);
		packed.set_entry(3, 4);

		assert_eq!(packed.get_entry(0), 1);
		assert_eq!(packed.get_entry(1), 2);
		assert_eq!(packed.get_entry(2), 3);
		assert_eq!(packed.get_entry(3), 4);
	}

	#[test]
	fn extract_from_hex() {
		let packed = PackedEntries {
			data: 0x0020863148418841,
			bpe: 5
		};

		assert_eq!(packed.get_entry(0), 1);
		assert_eq!(packed.get_entry(1), 2);
		assert_eq!(packed.get_entry(2), 2);
		assert_eq!(packed.get_entry(3), 3);
		assert_eq!(packed.get_entry(4), 4);
		assert_eq!(packed.get_entry(5), 4);
		assert_eq!(packed.get_entry(6), 5);
		assert_eq!(packed.get_entry(7), 6);
		assert_eq!(packed.get_entry(8), 6);
		assert_eq!(packed.get_entry(9), 4);
		assert_eq!(packed.get_entry(10), 8);

		let packed = PackedEntries {
			data: 0x01018A7260F68C87,
			bpe: 5
		};

		assert_eq!(packed.get_entry(0), 7);
		assert_eq!(packed.get_entry(1), 4);
		assert_eq!(packed.get_entry(2), 3);
		assert_eq!(packed.get_entry(3), 13);
		assert_eq!(packed.get_entry(4), 15);
		assert_eq!(packed.get_entry(5), 16);
		assert_eq!(packed.get_entry(6), 9);
		assert_eq!(packed.get_entry(7), 14);
		assert_eq!(packed.get_entry(8), 10);
		assert_eq!(packed.get_entry(9), 12);
		assert_eq!(packed.get_entry(10), 0);
		assert_eq!(packed.get_entry(11), 2);
	}
}