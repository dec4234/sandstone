//! Types found in game such as position, etc.

use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::serializer_error::SerializingErr;
use zerocopy::{FromBytes, FromZeroes};
use sandstone_derive::{McDeserialize, McSerialize};

/// A Minecraft position, internally represented as a 64-bit integer.
#[derive(McSerialize, McDeserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, FromBytes, FromZeroes, Clone)]
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