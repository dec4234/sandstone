//! World data structures

use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::game_types::Position;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

pub mod chunk;
pub mod generator;

#[derive(McDefault, McSerialize, McDeserialize, Debug, Copy, Clone)]
pub struct BlockPos {
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

impl BlockPos {
	pub fn new(x: i32, y: i32, z: i32) -> Self {
		Self {
			x,
			y,
			z,
		}
	}
}

impl From<Position> for BlockPos {
	fn from(pos: Position) -> Self {
		Self {
			x: pos.x() as i32,
			y: pos.y() as i32,
			z: pos.z() as i32,
		}
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Copy, Clone)]
pub struct ChunkPos {
	pub x: i32,
	pub z: i32,
}

impl ChunkPos {
	pub fn new(x: i32, z: i32) -> Self {
		Self {
			x,
			z,
		}
	}
}

impl From<BlockPos> for ChunkPos {
	fn from(pos: BlockPos) -> Self {
		Self {
			x: pos.x >> 4,
			z: pos.z >> 4,
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn chunk_pos_floors_negative_blocks() {
		// Block x = -1 is in chunk -1, not chunk 0. Integer division would round toward zero and
		// put the block in the wrong chunk.
		let cases = [(0, 0), (15, 0), (16, 1), (-1, -1), (-16, -1), (-17, -2)];
		for (block, chunk) in cases {
			let c = ChunkPos::from(BlockPos::new(block, 64, block));
			assert_eq!((c.x, c.z), (chunk, chunk), "block {block}");
		}
	}

	#[test]
	fn block_pos_from_position_keeps_sign() {
		let b = BlockPos::from(Position::new(-5, -64, 30_000_000));
		assert_eq!((b.x, b.y, b.z), (-5, -64, 30_000_000));
	}

	#[test]
	fn block_pos_is_three_ints() {
		let mut s = McSerializer::new();
		BlockPos::new(1, -1, 2).mc_serialize(&mut s).unwrap();
		assert_eq!(s.as_bytes(), &[0, 0, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 2]);

		let b = BlockPos::mc_deserialize(&mut McDeserializer::new(s.as_bytes())).unwrap();
		assert_eq!((b.x, b.y, b.z), (1, -1, 2));
	}
}
