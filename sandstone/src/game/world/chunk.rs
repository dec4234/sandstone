//! A collection of internal data formats for chunk storage
//!
//! https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format

use crate::game::world::chunk::PaletteFormatType::{BIOMES, BLOCKS};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol_types::datatypes::game_types::PackedEntries;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDeserialize, McSerialize};

// todo https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct ChunkSectionData {
	/// This array is NOT length-prefixed. The number of elements in the array is calculated based on the world's height. 
	/// Sections are sent bottom-to-top. The world height changes based on the dimension. 
	/// The height of each dimension is assigned by the server in its corresponding registry data entry. 
	/// For example, the vanilla overworld is 384 blocks tall, meaning 24 chunk sections will be included in this array
	pub data: Vec<ChunkSection>,
}

// convert the section data into a PrefixedArray<u8>
impl McSerialize for ChunkSectionData {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let mut small_serializer = McSerializer::new();
		
		self.data.mc_serialize(&mut small_serializer)?;
		
		let prefixed_array = PrefixedArray::new(small_serializer.output);
		prefixed_array.mc_serialize(serializer)?;
		
		Ok(())
	}
}

impl McDeserialize for ChunkSectionData {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let prefixed_array = PrefixedArray::<u8>::mc_deserialize(deserializer)?;
		
		let mut small_deserializer = McDeserializer::new(&prefixed_array.vec);
		
		let data = Vec::mc_deserialize(&mut small_deserializer)?;
		
		Ok(Self {
			data
		})
	}
}

#[derive(McSerialize, Debug, Clone, Hash, PartialEq)]
pub struct ChunkSection {
	/// Number of non-air blocks present in the chunk section. "Non-air" is defined as any fluid and block other than air, cave air, and void air
	pub block_count: i16,
	/// Consists of 4096 entries, representing all the blocks in the chunk section
	pub block_states: PalletedContainer,
	/// Consists of 64 entries, representing 4×4×4 biome regions in the chunk section
	pub biomes: PalletedContainer,
}

impl McDeserialize for ChunkSection {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let block_count = i16::mc_deserialize(deserializer)?;
		let block_states = PalletedContainer::mc_deserialize(deserializer, 4096, BLOCKS)?;
		let biomes = PalletedContainer::mc_deserialize(deserializer, 64, BIOMES)?;

		Ok(Self {
			block_count,
			block_states,
			biomes
		})
	}
}

#[derive(McSerialize, Debug, Clone, Hash, PartialEq)]
pub struct PalletedContainer {
	pub bits_per_entry: u8,
	pub palette: PalleteFormat,
	pub data: Vec<PackedEntries>
}

impl PalletedContainer {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer, num_entries: u16, typ: PaletteFormatType) -> SerializingResult<'a, Self> where Self: Sized {
		let bpe = u8::mc_deserialize(deserializer)?;
		let palette = PalleteFormat::mc_deserialize(deserializer, bpe, typ)?;
		
		let num_i64s = (num_entries as f32 / entries_per_i64(bpe) as f32).ceil() as u16;
		
		let mut data = Vec::with_capacity(num_i64s as usize);
		
		for _ in 0..num_i64s {
			data.push(PackedEntries::mc_deserialize(deserializer, bpe)?);
		}
		
		Ok(Self {
			bits_per_entry: bpe,
			palette,
			data
		})
	}
}

/// Used to determine which palette format to use based on the Bits Per Entry
#[derive(Debug, Clone, Hash, PartialEq)]
pub enum PaletteFormatType {
	BLOCKS,
	BIOMES
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub enum PalleteFormat {
	SingleValued(VarInt),
	Indirect(IndirectFormat),
	Direct
}

impl McSerialize for PalleteFormat {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		Ok(match self {
			PalleteFormat::SingleValued(value) => {
				value.mc_serialize(serializer)?
			}
			PalleteFormat::Indirect(format) => {
				format.mc_serialize(serializer)?
			}
			PalleteFormat::Direct => {
				// nothing
			}
		})
	}
}

impl PalleteFormat {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer, bits_per_entry: u8, typ: PaletteFormatType) -> SerializingResult<'a, Self> where Self: Sized {
		if bits_per_entry == 0 {
			return Ok(PalleteFormat::SingleValued(VarInt::mc_deserialize(deserializer)?));
		}
		
		match typ {
			BLOCKS => {
				if bits_per_entry >= 4 && bits_per_entry <= 8 {
					Ok(PalleteFormat::Indirect(IndirectFormat::mc_deserialize(deserializer)?))
				} else if bits_per_entry == 15 {
					Ok(PalleteFormat::Direct)
				} else {
					Err(SerializingErr::InvalidBitsPerEntry)
				}
			}
			BIOMES => {
				if bits_per_entry >= 1 && bits_per_entry <= 3 {
					Ok(PalleteFormat::Indirect(IndirectFormat::mc_deserialize(deserializer)?))
				} else if bits_per_entry == 6 {
					Ok(PalleteFormat::Direct)
				} else {
					Err(SerializingErr::InvalidBitsPerEntry)
				}
			}
		}
	}
}

#[derive(McSerialize, McDeserialize, Debug, Clone, Hash, PartialEq)]
pub struct IndirectFormat {
	pub length: VarInt,
	pub array: Vec<VarInt>
}

#[derive(McSerialize, McDeserialize, Debug, Clone, Hash, PartialEq)]
pub struct Heightmap {
	typ: VarInt,
	length: VarInt,
	data: Vec<i64>,
}

/// An entry is defined by a set of adjacent bits packed within the same long. The bits per entry value
/// differs based on a variety of factors (check wiki). This function calculates the number of entries that
/// can entirely fit within the same long. Entries cannot be split across longs, so any remaining bits are 
/// wasted as padding.
fn entries_per_i64(bpe: u8) -> u8 {
	if bpe == 0 {
		return 0;
	}
	
	64 / bpe
}

#[cfg(test)]
mod test {
	use crate::game::world::chunk::entries_per_i64;

	#[test]
	fn test_entries_per_i64() {
		assert_eq!(entries_per_i64(1), 64);
		assert_eq!(entries_per_i64(2), 32);
		assert_eq!(entries_per_i64(3), 21);
		assert_eq!(entries_per_i64(4), 16);
		assert_eq!(entries_per_i64(5), 12);
		assert_eq!(entries_per_i64(6), 10);
		assert_eq!(entries_per_i64(7), 9);
		assert_eq!(entries_per_i64(8), 8);
	}
}