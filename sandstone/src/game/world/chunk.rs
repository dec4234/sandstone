//! A collection of internal data formats for chunk storage
//!
//! https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDeserialize, McSerialize};
use crate::game::world::chunk::PaletteFormatType::{BIOMES, BLOCKS};

#[derive(McSerialize, McDeserialize, Debug, Clone, Hash, PartialEq)]
pub struct Chunk {
	/// This array is NOT length-prefixed. The number of elements in the array is calculated based on the world's height. 
	/// Sections are sent bottom-to-top. The world height changes based on the dimension. 
	/// The height of each dimension is assigned by the server in its corresponding registry data entry. 
	/// For example, the vanilla overworld is 384 blocks tall, meaning 24 chunk sections will be included in this array
	pub data: Vec<ChunkSection>,
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

/// Represents a packed i64 (long) that contains block or biome data. See 
/// https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Data_Array_format for more info. This
/// matches the spec for packed data after 1.16
#[derive(Debug, Clone, Hash, PartialEq)]
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer, bpe: u8) -> SerializingResult<'a, Self> where Self: Sized {
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

// todo
pub struct ChunkDataUpdateLight {
	
}

fn entries_per_i64(bpe: u8) -> u8 {
	if bpe == 0 {
		return 0;
	}
	
	64 / bpe
}

#[cfg(test)]
mod test {
	use crate::game::world::chunk::{entries_per_i64, PackedEntries};

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