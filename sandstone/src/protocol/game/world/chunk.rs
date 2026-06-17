//! A collection of internal data formats for chunk data transfer
//!
//! https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format

use crate::protocol::game::world::chunk::PaletteFormatType::{BIOMES, BLOCKS};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::internal_types::PackedEntries;
use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::util::java::bitset::BitSet;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

/// Chunk Data field as defined in https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ChunkData {
	pub heightmaps: PrefixedArray<Heightmap>,
	pub data: ChunkByteData,
	pub block_entities: PrefixedArray<BlockEntity>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct LightData {
	pub sky_light_mask: BitSet,
	pub block_light_mask: BitSet,
	pub empty_light_mask: BitSet,
	pub empty_block_light_mask: BitSet,
	pub sky_light: PrefixedArray<LightArray>,
	pub block_light: PrefixedArray<LightArray>,
}

/// The length of the inner array is always 2048; There is 1 array for each bit set to true in the block 
/// light mask, starting with the lowest value. Half a byte per light value. Acceptable light values are
/// 0-15
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct LightArray {
	pub data: PrefixedArray<u8>
}

impl LightArray {
	pub fn new() -> Self {
		Self {
			data: PrefixedArray::new(vec![0; 2048])
		}
	}

	/// Set the light value for the given index. Half a byte per light value.
	pub fn set(&mut self, index: usize, value: u8) -> SerializingResult<()> {
		if value > 0x0F {
			return Err(SerializingErr::OutOfBounds(format!("Light value {} is out of bounds. Valid values are 0-15", value)));
		}

		let byte_index = index / 2;
		let is_high = index % 2 == 1;

		if byte_index >= self.data.vec.len() {
			return Err(SerializingErr::OutOfBounds(format!("Light index {} is out of bounds. Valid indices are 0-4095", index)));
		}

		let byte = self.data.vec[byte_index];
		self.data.vec[byte_index] = if is_high {
			// Set high nibble
			(byte & 0x0F) | (value << 4)
		} else {
			// Set low nibble
			(byte & 0xF0) | (value & 0x0F)
		};
		
		Ok(())
	}
	
	/// Get the light value for the given index. Half a byte per light value.
	pub fn get(&self, index: usize) -> SerializingResult<u8> {
		let byte_index = index / 2;
		let is_high = index % 2 == 1;

		if byte_index >= self.data.vec.len() {
			return Err(SerializingErr::OutOfBounds(format!("Light index {} is out of bounds. Valid indices are 0-4095", index)));
		}

		let byte = self.data.vec[byte_index];
		Ok(if is_high {
			byte >> 4
		} else {
			byte & 0x0F
		})
	}
}

/// An array of 24 chunk sections, containing the block data for a single chunk. This is serialized to/from
/// a byte array.
#[derive(McDefault, Debug, Clone, Hash, PartialEq)]
pub struct ChunkByteData {
	/// This array is NOT length-prefixed. The number of elements in the array is calculated based on the world's height. 
	/// Sections are sent bottom-to-top. The world height changes based on the dimension. 
	/// The height of each dimension is assigned by the server in its corresponding registry data entry. 
	/// For example, the vanilla overworld is 384 blocks tall, meaning 24 chunk sections will be included in this array
	pub data: Vec<ChunkSection>,
}

// convert the section data into a PrefixedArray<u8>
impl McSerialize for ChunkByteData {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let mut small_serializer = McSerializer::new();
		
		self.data.mc_serialize(&mut small_serializer)?;
		
		let prefixed_array = PrefixedArray::new(small_serializer.output);
		prefixed_array.mc_serialize(serializer)?;
		
		Ok(())
	}
}

impl McDeserialize for ChunkByteData {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let prefixed_array = PrefixedArray::<u8>::mc_deserialize(deserializer)?;
		
		let mut small_deserializer = McDeserializer::new(&prefixed_array.vec);
		
		let data = Vec::mc_deserialize(&mut small_deserializer)?;
		
		Ok(Self {
			data
		})
	}
}

#[derive(McDefault, McSerialize, Debug, Clone, Hash, PartialEq)]
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

/// Same as [ChunkSection] but only contains the biome data.
#[derive(McDefault, McSerialize, Debug, Clone, Hash, PartialEq)]
pub struct BiomeSection {
	/// Consists of 64 entries, representing 4×4×4 biome regions in the chunk section
	pub biomes: PalletedContainer
}

impl McDeserialize for BiomeSection {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		Ok(Self {
			biomes: PalletedContainer::mc_deserialize(deserializer, 64, BIOMES)?
		})
	}
}

/// Same as [ChunkByteData] but the sections contain only the biome data. Used by the Chunk Biomes
/// packet, which is serialized to/from a length-prefixed byte array.
#[derive(McDefault, Debug, Clone, PartialEq)]
pub struct BiomeByteData {
	/// This array is NOT length-prefixed. The number of elements matches the number of chunk sections
	/// in the dimension. Sections are sent bottom-to-top.
	pub data: Vec<BiomeSection>,
}

// convert the section data into a PrefixedArray<u8>
impl McSerialize for BiomeByteData {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let mut small_serializer = McSerializer::new();

		self.data.mc_serialize(&mut small_serializer)?;

		let prefixed_array = PrefixedArray::new(small_serializer.output);
		prefixed_array.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for BiomeByteData {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let prefixed_array = PrefixedArray::<u8>::mc_deserialize(deserializer)?;

		let mut small_deserializer = McDeserializer::new(&prefixed_array.vec);

		let data = Vec::mc_deserialize(&mut small_deserializer)?;

		Ok(Self {
			data
		})
	}
}

#[derive(McDefault, McSerialize, Debug, Clone, Hash, PartialEq)]
pub struct PalletedContainer {
	pub bits_per_entry: u8,
	pub palette: PalleteFormat,
	pub data: Vec<PackedEntries>
}

impl PalletedContainer {
	/// Build a single-valued container where every entry is the same value (bits per entry of 0).
	/// Used for all-air block sections and single-biome sections.
	pub fn single_valued(value: VarInt) -> Self {
		Self {
			bits_per_entry: 0,
			palette: PalleteFormat::SingleValued(value),
			data: vec![],
		}
	}

	/// Build an indirect container from a palette of distinct values and the per-entry indices into
	/// that palette. `indices.len()` is the number of entries in the section (4096 for blocks, 64
	/// for biomes). `typ` selects the valid bits-per-entry range. Errors if the palette is too large
	/// to be represented in the indirect range for `typ` (the caller should use the direct format
	/// instead in that case).
	pub fn indirect<'a>(palette: Vec<VarInt>, indices: &[u16], typ: PaletteFormatType) -> SerializingResult<'a, Self> {
		if palette.is_empty() {
			return Err(SerializingErr::OutOfBounds("Indirect palette must contain at least one entry".to_string()));
		}

		// smallest bpe that can index the palette
		let needed = (usize::BITS - (palette.len() - 1).leading_zeros()).max(1) as u8;
		let (min_bpe, max_bpe) = match typ {
			BLOCKS => (4u8, 8u8),
			BIOMES => (1u8, 3u8),
		};
		if needed > max_bpe {
			return Err(SerializingErr::InvalidBitsPerEntry);
		}
		let bpe = needed.max(min_bpe);

		let per_long = entries_per_i64(bpe);
		let mut data: Vec<PackedEntries> = Vec::new();
		for (i, &index) in indices.iter().enumerate() {
			let long_index = i / per_long as usize;
			let slot = (i % per_long as usize) as u8;
			if long_index >= data.len() {
				data.push(PackedEntries::new(bpe));
			}
			data[long_index].set_entry(slot, index as u64);
		}

		Ok(Self {
			bits_per_entry: bpe,
			palette: PalleteFormat::Indirect(IndirectFormat { palette: PrefixedArray::new(palette) }),
			data,
		})
	}

	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer, num_entries: u16, typ: PaletteFormatType) -> SerializingResult<'a, Self> where Self: Sized {
		let bpe = u8::mc_deserialize(deserializer)?;
		let palette = PalleteFormat::mc_deserialize(deserializer, bpe, typ)?;

		let data = if bpe == 0 {
			Vec::new()
		} else {
			let num_i64s = (num_entries as f32 / entries_per_i64(bpe) as f32).ceil() as u16;
			let mut data = Vec::with_capacity(num_i64s as usize);
			for _ in 0..num_i64s {
				data.push(PackedEntries::mc_deserialize(deserializer, bpe)?);
			}
			data
		};

		Ok(Self {
			bits_per_entry: bpe,
			palette,
			data
		})
	}
}

/// Used to determine which palette format to use based on the Bits Per Entry
#[derive(McDefault, Debug, Clone, Hash, PartialEq)]
pub enum PaletteFormatType {
	BLOCKS,
	BIOMES
}

#[derive(McDefault, Debug, Clone, Hash, PartialEq)]
pub enum PalleteFormat {
	SingleValued(VarInt),
	Indirect(IndirectFormat),
	Direct
}

impl McSerialize for PalleteFormat {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
  			PalleteFormat::SingleValued(value) => {
  				value.mc_serialize(serializer)?
  			}
  			PalleteFormat::Indirect(format) => {
  				format.mc_serialize(serializer)?
  			}
  			PalleteFormat::Direct => {
  				// nothing
  			}
  		};
		
  		Ok(())
	}
}

impl PalleteFormat {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer, bits_per_entry: u8, typ: PaletteFormatType) -> SerializingResult<'a, Self> where Self: Sized {
		if bits_per_entry == 0 {
			return Ok(PalleteFormat::SingleValued(VarInt::mc_deserialize(deserializer)?));
		}
		
		match typ {
			BLOCKS => {
				if (4..=8).contains(&bits_per_entry) {
					Ok(PalleteFormat::Indirect(IndirectFormat::mc_deserialize(deserializer)?))
				} else if bits_per_entry == 15 {
					Ok(PalleteFormat::Direct)
				} else {
					Err(SerializingErr::InvalidBitsPerEntry)
				}
			}
			BIOMES => {
				if (1..=3).contains(&bits_per_entry) {
					Ok(PalleteFormat::Indirect(IndirectFormat::mc_deserialize(deserializer)?))
				} else if bits_per_entry == 7 {
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
	pub palette: PrefixedArray<VarInt>
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, Hash, PartialEq)]
pub struct Heightmap {
	typ: VarInt,
	data: PrefixedArray<i64>,
}

impl Heightmap {
	pub fn new(typ: VarInt, data: PrefixedArray<i64>) -> Self {
		Self { typ, data }
	}
}

/// A block entity is something like a chest or other block which has NBT.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BlockEntity {
	pub packed_xz: PackedXZ,
	pub y: i16,
	pub typ: VarInt,
	pub data: NbtCompound
}

/// Relative coordinates within a chunk. Each x and z value has valid values 0-15
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PackedXZ {
	data: u8,
}

impl PackedXZ {
	pub fn new<'a>(x: u8, z: u8) -> SerializingResult<'a, Self> {
		if x > 15 || z > 15 {
			return Err(SerializingErr::OutOfBounds(format!("PackedXZ values out of bounds. Valid values are 0-15. Received x={}, z={}", x, z)));
		}
		
		Ok(Self {
			data: (x << 4) | z
		})
	}
	
	pub fn x(&self) -> u8 {
		self.data >> 4
	}
	
	pub fn z(&self) -> u8 {
		self.data & 0x0F
	}
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
	use crate::protocol::game::world::chunk::entries_per_i64;

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
	fn test_paletted_container_builders_round_trip() {
		use crate::protocol::game::world::chunk::{ChunkSection, PaletteFormatType, PalletedContainer};
		use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
		use crate::protocol_types::datatypes::var_types::VarInt;

		// 4 distinct block states spread across the 4096 entries, plus a single-valued biome.
		let palette = vec![VarInt(0), VarInt(1), VarInt(10), VarInt(9)];
		let mut indices = vec![0u16; 4096];
		for i in 0..256 {
			indices[i] = 1; // bottom layer -> palette index 1
		}
		let block_states = PalletedContainer::indirect(palette, &indices, PaletteFormatType::BLOCKS).unwrap();
		let biomes = PalletedContainer::single_valued(VarInt(40));

		let section = ChunkSection {
			block_count: 256,
			block_states,
			biomes,
		};

		// The property we care about: the builder produces bytes the existing deserializer accepts
		// and reconstructs identically.
		let mut serializer = McSerializer::new();
		section.mc_serialize(&mut serializer).unwrap();
		let bytes: Vec<u8> = serializer.into();
		let mut deserializer = McDeserializer::new(&bytes);
		let result = ChunkSection::mc_deserialize(&mut deserializer).unwrap();

		assert_eq!(section, result);
		assert!(deserializer.is_at_end());
	}

	#[test]
	fn test_light_array() {
		let mut light_array = super::LightArray::new();
		
		light_array.set(0, 0x0F).unwrap();
		assert_eq!(light_array.get(0).unwrap(), 0x0F);
		
		light_array.set(1, 0x0F).unwrap();
		assert_eq!(light_array.get(1).unwrap(), 0x0F);
		
		light_array.set(2, 0x00).unwrap();
		assert_eq!(light_array.get(2).unwrap(), 0x00);
		
		light_array.set(3, 0x00).unwrap();
		assert_eq!(light_array.get(3).unwrap(), 0x00);
		
		assert_eq!(light_array.get(4).is_err(), false);
	}
}