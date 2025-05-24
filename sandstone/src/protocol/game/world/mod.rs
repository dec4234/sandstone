//! World data structures

use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use sandstone_derive::{McDeserialize, McSerialize};
use crate::protocol_types::datatypes::game_types::Position;

pub mod chunk;

#[derive(McSerialize, McDeserialize, Debug, Copy, Clone)]
pub struct BlockPos {
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

impl BlockPos {
	pub fn new(x: i32, y: i32, z: i32) -> Self {
		Self { x, y, z }
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

#[derive(McSerialize, McDeserialize, Debug, Copy, Clone)]
pub struct ChunkPos {
	pub x: i32,
	pub z: i32,
}

impl ChunkPos {
	pub fn new(x: i32, z: i32) -> Self {
		Self { x, z }
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