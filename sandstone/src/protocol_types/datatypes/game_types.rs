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

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub enum GameDifficulty {
	Peaceful,
	Easy,
	Normal,
	Hard
}

impl McSerialize for GameDifficulty {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let value: u8 = match self {
			GameDifficulty::Peaceful => 0,
			GameDifficulty::Easy => 1,
			GameDifficulty::Normal => 2,
			GameDifficulty::Hard => 3
		};
		value.mc_serialize(serializer)
	}
}

impl McDeserialize for GameDifficulty {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized
	{
		let value = u8::mc_deserialize(deserializer)?;
		let difficulty = match value {
			0 => GameDifficulty::Peaceful,
			1 => GameDifficulty::Easy,
			2 => GameDifficulty::Normal,
			3 => GameDifficulty::Hard,
			_ => return Err(SerializingErr::DeserializationError("Invalid game difficulty value".to_string()))
		};
		Ok(difficulty)
	}
}

impl McDefault for GameDifficulty {
	fn mc_default() -> Self {
		GameDifficulty::Normal
	}
}

#[cfg(test)]
mod test {
	use crate::protocol_types::datatypes::game_types::Position;

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
}