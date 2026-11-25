//! Read and write Minecraft region files (the Anvil format), and change their chunks to and from
//! `Chunk Data and Update Light` packets.
//!
//! - [RegionFile] reads and writes the NBT of each chunk in one `r.<x>.<z>.mca` file. It supports
//!   gzip, zlib, no compression, LZ4, and chunks in external `.mcc` files.
//! - [Chunk] is the typed form of that NBT. A read and a write keep the tags that it does not know.
//! - [Chunk::to_packet] and [Chunk::from_packet] change a chunk to and from the packet. They need a
//!   [Dimension] for the world height and the biome registry order.
//!
//! The block state names and IDs are from the vanilla 26.3 server ([DATA_VERSION]). A chunk from
//! another version can have block names or properties that this version does not know. The
//! conversion gives an error for them.
//!
//! ```no_run
//! use sandstone::protocol::game::world::anvil::{Dimension, RegionFile};
//!
//! let mut region = RegionFile::open("world/region/r.0.0.mca").unwrap();
//! if let Some(chunk) = region.read_chunk(3, 5).unwrap() {
//!     let packet = chunk.to_packet(&Dimension::overworld()).unwrap();
//! }
//! ```

mod block_state;
mod chunk;
mod convert;
mod lz4;
mod region;

pub use block_state::BlockState;
pub use chunk::{BIOME_MIN_BITS, BLOCK_MIN_BITS, Chunk, DATA_VERSION, PalettedData, Section};
pub use convert::{Dimension, vanilla_biomes};
pub use region::{Compression, RegionFile, region_file_name};

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnvilError {
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error(transparent)]
	Serializing(#[from] SerializingErr),
	#[error(transparent)]
	Nbt(#[from] NbtError),
	#[error("Invalid chunk data: {0}")]
	InvalidChunk(String),
	#[error("Unknown block state: {0}")]
	UnknownBlock(String),
	#[error("Unknown biome: {0}")]
	UnknownBiome(String),
	#[error("Unsupported compression type {0}")]
	UnsupportedCompression(u8),
}

pub type AnvilResult<T> = Result<T, AnvilError>;
