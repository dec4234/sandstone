//! A collection of internal data formats for chunk data transfer
//!
//! https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format

use crate::protocol::game::world::block_state_table::BLOCK_STATE_COUNT;
use crate::protocol::game::world::chunk::PaletteFormatType::{BIOMES, BLOCKS};
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::internal_types::PackedEntries;
use crate::protocol_types::datatypes::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::util::java::bitset::BitSet;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

/// Chunk Data field as defined in https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data_and_Update_Light
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ChunkData {
	pub heightmaps: PrefixedArray<Heightmap>,
	pub data: ChunkByteData,
	pub block_entities: PrefixedArray<BlockEntity>,
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Data_types#Light_Data>
///
/// The wiki says the four masks are BitSets (a VarInt count of longs, then the longs). A 26.3
/// vanilla server sends each mask as a Prefixed Array of Byte (Java's `BitSet.toByteArray`)
/// instead, so the masks are serialized by hand here.
#[derive(McDefault, Debug, Clone, PartialEq)]
pub struct LightData {
	pub sky_light_mask: BitSet,
	pub block_light_mask: BitSet,
	pub empty_light_mask: BitSet,
	pub empty_block_light_mask: BitSet,
	pub sky_light: PrefixedArray<LightArray>,
	pub block_light: PrefixedArray<LightArray>,
}

impl McSerialize for LightData {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		for mask in [&self.sky_light_mask, &self.block_light_mask, &self.empty_light_mask, &self.empty_block_light_mask] {
			PrefixedArray::<u8>::new(mask.clone().into()).mc_serialize(serializer)?;
		}
		self.sky_light.mc_serialize(serializer)?;
		self.block_light.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for LightData {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		fn read_mask<'a>(deserializer: &mut McDeserializer) -> SerializingResult<'a, BitSet> {
			let bytes = PrefixedArray::<u8>::mc_deserialize(deserializer)?;
			Ok(BitSet::from(bytes.vec.as_slice()))
		}

		Ok(Self {
			sky_light_mask: read_mask(deserializer)?,
			block_light_mask: read_mask(deserializer)?,
			empty_light_mask: read_mask(deserializer)?,
			empty_block_light_mask: read_mask(deserializer)?,
			sky_light: PrefixedArray::mc_deserialize(deserializer)?,
			block_light: PrefixedArray::mc_deserialize(deserializer)?,
		})
	}
}

/// The length of the inner array is always 2048; There is 1 array for each bit set to true in the block
/// light mask, starting with the lowest value. Half a byte per light value. Acceptable light values are
/// 0-15
#[derive(McDefault, McSerialize, McDeserialize, Default, Debug, Clone, PartialEq)]
pub struct LightArray {
	pub data: PrefixedArray<u8>,
}

impl LightArray {
	pub fn new() -> Self {
		Self {
			data: PrefixedArray::new(vec![0; 2048]),
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let prefixed_array = PrefixedArray::<u8>::mc_deserialize(deserializer)?;

		let mut small_deserializer = McDeserializer::new(&prefixed_array.vec);

		let data = Vec::mc_deserialize(&mut small_deserializer)?;

		Ok(Self {
			data,
		})
	}
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Chunk_Section_structure>
#[derive(McDefault, McSerialize, Debug, Clone, Hash, PartialEq)]
pub struct ChunkSection {
	/// Number of non-air blocks present in the chunk section. "Non-air" is defined as any fluid and block other than air, cave air, and void air
	pub block_count: i16,
	/// Number of waterlogged blocks plus `minecraft:water` or `minecraft:lava` blocks in any state
	pub fluid_count: i16,
	/// Consists of 4096 entries, representing all the blocks in the chunk section
	pub block_states: PalletedContainer,
	/// Consists of 64 entries, representing 4×4×4 biome regions in the chunk section
	pub biomes: PalletedContainer,
}

impl McDeserialize for ChunkSection {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let block_count = i16::mc_deserialize(deserializer)?;
		let fluid_count = i16::mc_deserialize(deserializer)?;
		let block_states = PalletedContainer::mc_deserialize(deserializer, 4096, BLOCKS)?;
		let biomes = PalletedContainer::mc_deserialize(deserializer, 64, BIOMES)?;

		Ok(Self {
			block_count,
			fluid_count,
			block_states,
			biomes,
		})
	}
}

/// Same as [ChunkSection] but only contains the biome data.
#[derive(McDefault, McSerialize, Debug, Clone, Hash, PartialEq)]
pub struct BiomeSection {
	/// Consists of 64 entries, representing 4×4×4 biome regions in the chunk section
	pub biomes: PalletedContainer,
}

impl McDeserialize for BiomeSection {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		Ok(Self {
			biomes: PalletedContainer::mc_deserialize(deserializer, 64, BIOMES)?,
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let prefixed_array = PrefixedArray::<u8>::mc_deserialize(deserializer)?;

		let mut small_deserializer = McDeserializer::new(&prefixed_array.vec);

		let data = Vec::mc_deserialize(&mut small_deserializer)?;

		Ok(Self {
			data,
		})
	}
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Paletted_Container_structure>
#[derive(McSerialize, Debug, Clone, Hash, PartialEq)]
pub struct PalletedContainer {
	pub bits_per_entry: u8,
	pub palette: PalleteFormat,
	pub data: Vec<PackedEntries>,
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
			palette: PalleteFormat::Indirect(IndirectFormat {
				palette: PrefixedArray::new(palette),
			}),
			data,
		})
	}

	/// Build the smallest container for `values` (global IDs, one per entry). One distinct value gives
	/// a single-valued container. A palette that fits the indirect range gives an indirect container.
	/// Other values use the direct format with [`direct_bits_per_entry`] bits.
	pub fn from_values<'a>(values: &[i32], typ: PaletteFormatType) -> SerializingResult<'a, Self> {
		let mut palette: Vec<i32> = Vec::new();
		let mut indices = Vec::with_capacity(values.len());
		for &value in values {
			let index = match palette.iter().position(|&v| v == value) {
				Some(index) => index,
				None => {
					palette.push(value);
					palette.len() - 1
				}
			};
			indices.push(index as u16);
		}

		match palette.len() {
			0 => Err(SerializingErr::OutOfBounds("A paletted container needs at least one entry".to_string())),
			1 => Ok(Self::single_valued(VarInt(palette[0]))),
			_ => match Self::indirect(palette.into_iter().map(VarInt).collect(), &indices, typ) {
				Err(SerializingErr::InvalidBitsPerEntry) => Self::direct(values, typ),
				other => other,
			},
		}
	}

	/// Build a direct container. Each entry holds its global ID.
	fn direct<'a>(values: &[i32], typ: PaletteFormatType) -> SerializingResult<'a, Self> {
		let bpe = direct_bits_per_entry(&typ);
		let per_long = entries_per_i64(bpe) as usize;
		let mut data = vec![PackedEntries::new(bpe); values.len().div_ceil(per_long)];
		for (i, &value) in values.iter().enumerate() {
			if value < 0 || value as u64 >= 1 << bpe {
				return Err(SerializingErr::OutOfBounds(format!("ID {value} does not fit in a direct palette of {bpe} bits")));
			}
			data[i / per_long].set_entry((i % per_long) as u8, value as u64);
		}

		Ok(Self {
			bits_per_entry: bpe,
			palette: PalleteFormat::Direct,
			data,
		})
	}

	/// The global ID of each of the `num_entries` entries (4096 for blocks, 64 for biomes).
	pub fn values<'a>(&self, num_entries: usize) -> SerializingResult<'a, Vec<i32>> {
		if let PalleteFormat::SingleValued(value) = &self.palette {
			return Ok(vec![value.0; num_entries]);
		}

		let per_long = entries_per_i64(self.bits_per_entry) as usize;
		if per_long == 0 || self.data.len() < num_entries.div_ceil(per_long) {
			return Err(SerializingErr::InconsistentField(format!(
				"{} longs is too few for {num_entries} entries at {} bits",
				self.data.len(),
				self.bits_per_entry
			)));
		}

		(0..num_entries)
			.map(|i| {
				let entry = self.data[i / per_long].get_entry((i % per_long) as u8);
				match &self.palette {
					PalleteFormat::Indirect(format) => format
						.palette
						.vec
						.get(entry as usize)
						.map(|id| id.0)
						.ok_or_else(|| SerializingErr::OutOfBounds(format!("Palette index {entry} is out of bounds"))),
					_ => Ok(entry as i32),
				}
			})
			.collect()
	}

	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer, num_entries: u16, typ: PaletteFormatType) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
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
			data,
		})
	}
}

/// A derived default would give a bits per entry of `u8::mc_default()`, which no reader accepts.
/// A single-valued container is valid for both block states and biomes.
impl McDefault for PalletedContainer {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		Self::single_valued(VarInt::mc_default())
	}
}

/// Used to determine which palette format to use based on the Bits Per Entry
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Palette_formats>
#[derive(McDefault, Debug, Clone, Copy, Hash, PartialEq)]
pub enum PaletteFormatType {
	BLOCKS,
	BIOMES,
}

#[derive(McDefault, Debug, Clone, Hash, PartialEq)]
pub enum PalleteFormat {
	SingleValued(VarInt),
	Indirect(IndirectFormat),
	Direct,
}

impl McSerialize for PalleteFormat {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			PalleteFormat::SingleValued(value) => value.mc_serialize(serializer)?,
			PalleteFormat::Indirect(format) => format.mc_serialize(serializer)?,
			PalleteFormat::Direct => {
				// nothing
			}
		};

		Ok(())
	}
}

impl PalleteFormat {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer, bits_per_entry: u8, typ: PaletteFormatType) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		if bits_per_entry == 0 {
			return Ok(PalleteFormat::SingleValued(VarInt::mc_deserialize(deserializer)?));
		}

		match typ {
			BLOCKS => {
				if (4..=8).contains(&bits_per_entry) {
					Ok(PalleteFormat::Indirect(IndirectFormat::mc_deserialize(deserializer)?))
				} else if bits_per_entry == direct_bits_per_entry(&BLOCKS) {
					Ok(PalleteFormat::Direct)
				} else {
					Err(SerializingErr::InvalidBitsPerEntry)
				}
			}
			BIOMES => {
				if (1..=3).contains(&bits_per_entry) {
					Ok(PalleteFormat::Indirect(IndirectFormat::mc_deserialize(deserializer)?))
				} else if bits_per_entry == direct_bits_per_entry(&BIOMES) {
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
	pub palette: PrefixedArray<VarInt>,
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Chunk_format#Heightmap_structure>
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, Hash, PartialEq)]
pub struct Heightmap {
	typ: VarInt,
	data: PrefixedArray<i64>,
}

impl Heightmap {
	pub fn new(typ: VarInt, data: PrefixedArray<i64>) -> Self {
		Self {
			typ,
			data,
		}
	}

	/// The heightmap type ID (for example 4 is `MOTION_BLOCKING`).
	pub fn typ(&self) -> VarInt {
		self.typ
	}

	/// The packed heights, one entry per column.
	pub fn data(&self) -> &[i64] {
		&self.data.vec
	}
}

/// A block entity is something like a chest or other block which has NBT.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BlockEntity {
	pub packed_xz: PackedXZ,
	pub y: i16,
	pub typ: VarInt,
	pub data: NbtCompound,
}

/// Relative coordinates within a chunk. Each x and z value has valid values 0-15
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct PackedXZ {
	data: u8,
}

impl PackedXZ {
	pub fn new<'a>(x: u8, z: u8) -> SerializingResult<'a, Self> {
		if x > 15 || z > 15 {
			return Err(SerializingErr::OutOfBounds(format!("PackedXZ values out of bounds. Valid values are 0-15. Received x={}, z={}", x, z)));
		}

		Ok(Self {
			data: (x << 4) | z,
		})
	}

	pub fn x(&self) -> u8 {
		self.data >> 4
	}

	pub fn z(&self) -> u8 {
		self.data & 0x0F
	}
}

/// The bits per entry of a direct palette: `ceil(log2(number of IDs))`. The client computes it from
/// the registry size. For blocks that is [`BLOCK_STATE_COUNT`] (35723 states in 26.3 -> 16 bits, not
/// the 15 that older protocol versions used). For biomes it is 7, from the 67 vanilla biomes. A server
/// with a different number of biomes needs a different value.
pub fn direct_bits_per_entry(typ: &PaletteFormatType) -> u8 {
	match typ {
		BLOCKS => (u32::BITS - (BLOCK_STATE_COUNT as u32 - 1).leading_zeros()) as u8,
		BIOMES => 7,
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
		indices[..256].fill(1); // bottom layer -> palette index 1
		let block_states = PalletedContainer::indirect(palette, &indices, PaletteFormatType::BLOCKS).unwrap();
		let biomes = PalletedContainer::single_valued(VarInt(40));

		let section = ChunkSection {
			block_count: 256,
			fluid_count: 0,
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

    use crate::protocol::game::world::chunk::{BiomeByteData, BiomeSection, ChunkSection, PackedXZ, PaletteFormatType, PalleteFormat, PalletedContainer};
    use crate::protocol::serialization::serializer_error::SerializingErr;
    use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
    use crate::protocol_types::datatypes::internal_types::PackedEntries;
    use crate::protocol_types::datatypes::var_types::VarInt;

    fn to_bytes<T: McSerialize>(value: &T) -> Vec<u8> {
		let mut serializer = McSerializer::new();
		value.mc_serialize(&mut serializer).unwrap();
		serializer.output
	}

	/// The Chunk Biomes packet wraps biome-only sections in a length-prefixed byte array. The
	/// reader must use the biome rules (64 entries, indirect 1-3 bpe) for each section.
	#[test]
	fn biome_byte_data_round_trip() {
		let indices: Vec<u16> = (0..64).map(|i| (i % 3) as u16).collect();
		let indirect = PalletedContainer::indirect(vec![VarInt(4), VarInt(5), VarInt(6)], &indices, PaletteFormatType::BIOMES).unwrap();
		assert_eq!(indirect.bits_per_entry, 2);
		assert_eq!(indirect.data.len(), 2, "64 entries at 32 per long");

		let data = BiomeByteData {
			data: vec![BiomeSection { biomes: PalletedContainer::single_valued(VarInt(1)) }, BiomeSection { biomes: indirect }],
		};
		let bytes = to_bytes(&data);
		// Byte array length prefix, then section 1 (bpe 0, value 1), then section 2 (bpe 2, ...).
		assert_eq!(bytes[0] as usize, bytes.len() - 1);
		assert_eq!(&bytes[1..4], &[0, 1, 2]);

		let mut de = McDeserializer::new(&bytes);
		assert_eq!(BiomeByteData::mc_deserialize(&mut de).unwrap(), data);
		assert!(de.is_at_end());
	}

	/// Direct format has no palette on the wire: bpe, then the packed longs. In 26.3 blocks use 16 bpe
	/// (35723 states) and biomes use 7 bpe (67 biomes).
	#[test]
	fn direct_palette_round_trip() {
		assert_eq!(super::direct_bits_per_entry(&PaletteFormatType::BLOCKS), 16);
		for (typ, bpe, entries) in [(PaletteFormatType::BLOCKS, 16u8, 4096u16), (PaletteFormatType::BIOMES, 7, 64)] {
			let longs = (entries as usize).div_ceil((64 / bpe) as usize);
			let container = PalletedContainer {
				bits_per_entry: bpe,
				palette: PalleteFormat::Direct,
				data: vec![PackedEntries::new(bpe); longs],
			};
			let bytes = to_bytes(&container);
			assert_eq!(bytes.len(), 1 + longs * 8, "{typ:?}: no palette bytes");

			let mut de = McDeserializer::new(&bytes);
			assert_eq!(PalletedContainer::mc_deserialize(&mut de, entries, typ).unwrap(), container);
			assert!(de.is_at_end());
		}
	}

	/// A bpe outside the valid ranges must be an error, not a guess. Biome ranges are smaller than
	/// block ranges, so a value that is valid for blocks can be invalid for biomes.
	#[test]
	fn invalid_bits_per_entry_errors() {
		// 15 was the direct value before the block state count went over 32768.
		for (typ, bpe) in [
			(PaletteFormatType::BLOCKS, 3u8),
			(PaletteFormatType::BLOCKS, 9),
			(PaletteFormatType::BLOCKS, 15),
			(PaletteFormatType::BLOCKS, 17),
			(PaletteFormatType::BIOMES, 4),
			(PaletteFormatType::BIOMES, 8),
		] {
			let bytes = [bpe, 0, 0, 0];
			let result = PalletedContainer::mc_deserialize(&mut McDeserializer::new(&bytes), 64, typ);
			assert!(matches!(result, Err(SerializingErr::InvalidBitsPerEntry)), "{typ:?} bpe {bpe}: {result:?}");
		}
	}

	/// The indirect builder must refuse an empty palette and a palette too large for the indirect range.
	#[test]
	fn indirect_builder_errors() {
		assert!(matches!(PalletedContainer::indirect(vec![], &[0; 64], PaletteFormatType::BIOMES), Err(SerializingErr::OutOfBounds(_))));

		// 9 entries need 4 bits, but the biome indirect range stops at 3.
		let palette: Vec<VarInt> = (0..9).map(VarInt).collect();
		assert!(matches!(PalletedContainer::indirect(palette, &[0; 64], PaletteFormatType::BIOMES), Err(SerializingErr::InvalidBitsPerEntry)));

		// A 2-entry block palette still uses the minimum of 4 bpe.
		let small = PalletedContainer::indirect(vec![VarInt(0), VarInt(1)], &[0; 4096], PaletteFormatType::BLOCKS).unwrap();
		assert_eq!(small.bits_per_entry, 4);
	}

	/// The mask bytes come from a chunk packet sent by a 26.3 vanilla server. Each mask is a
	/// Prefixed Array of Byte, not a count of longs. With the long layout, the reader takes the
	/// wrong number of light arrays, and some chunk packets fail with "Input ended prematurely".
	#[test]
	fn light_data_reads_byte_array_masks_from_wire() {
        use crate::protocol::game::world::chunk::LightData;

        let mut bytes = vec![2, 0x00, 0x06, 1, 0xFE, 2, 0xFF, 0x01, 2, 0x01, 0x07];
		for count in [2, 7] {
			bytes.push(count);
			for _ in 0..count {
				bytes.extend_from_slice(&[0x80, 0x10]); // VarInt 2048
				bytes.extend_from_slice(&[0; 2048]);
			}
		}

		let mut de = McDeserializer::new(&bytes);
		let light = LightData::mc_deserialize(&mut de).unwrap();
		assert!(de.is_at_end());

		let set_bits = |mask: &crate::util::java::bitset::BitSet| (0..64).filter(|&i| mask.get_bit(i)).collect::<Vec<_>>();
		assert_eq!(set_bits(&light.sky_light_mask), vec![9, 10]);
		assert_eq!(set_bits(&light.block_light_mask), vec![1, 2, 3, 4, 5, 6, 7]);
		assert_eq!(set_bits(&light.empty_light_mask), vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
		assert_eq!(set_bits(&light.empty_block_light_mask), vec![0, 8, 9, 10]);
		// one light array per set bit in the matching mask
		assert_eq!(light.sky_light.vec.len(), 2);
		assert_eq!(light.block_light.vec.len(), 7);

		// writing it back gives the same bytes
		let mut serializer = McSerializer::new();
		light.mc_serialize(&mut serializer).unwrap();
		assert_eq!(serializer.output, bytes);
	}

	/// 26.3 sends a fluid count after the block count. If the reader skips it, the fluid count
	/// bytes are read as the block states bits per entry and every chunk fails to decode.
	#[test]
	fn chunk_section_reads_fluid_count_from_wire() {
		// block count 5, fluid count 3, single-valued blocks (id 1), single-valued biomes (id 2)
		let bytes = [0x00, 0x05, 0x00, 0x03, 0x00, 0x01, 0x00, 0x02];
		let mut de = McDeserializer::new(&bytes);
		let section = ChunkSection::mc_deserialize(&mut de).unwrap();
		assert_eq!(section.block_count, 5);
		assert_eq!(section.fluid_count, 3);
		assert_eq!(section.block_states, PalletedContainer::single_valued(VarInt(1)));
		assert_eq!(section.biomes, PalletedContainer::single_valued(VarInt(2)));
		assert!(de.is_at_end());
	}

	/// A ChunkSection reads blocks with 4096 entries and then biomes with 64 entries. Mixing up
	/// the two counts would read the wrong number of longs.
	#[test]
	fn chunk_section_direct_blocks_then_biomes() {
		let section = ChunkSection {
			block_count: 1,
			fluid_count: 0,
			block_states: PalletedContainer {
				bits_per_entry: 16,
				palette: PalleteFormat::Direct,
				data: vec![PackedEntries::new(16); 4096usize.div_ceil(4)],
			},
			biomes: PalletedContainer::single_valued(VarInt(2)),
		};
		let bytes = to_bytes(&section);
		let mut de = McDeserializer::new(&bytes);
		assert_eq!(ChunkSection::mc_deserialize(&mut de).unwrap(), section);
		assert!(de.is_at_end());
	}

	/// `from_values` must pick the smallest valid format, and `values` must give back the same IDs.
	/// The Anvil conversion depends on both: a wrong format makes the client reject the chunk, and a
	/// wrong value puts the wrong block in the world.
	#[test]
	fn from_values_picks_format_and_round_trips() {
		let one = vec![7; 4096];
		let c = PalletedContainer::from_values(&one, PaletteFormatType::BLOCKS).unwrap();
		assert_eq!(c, PalletedContainer::single_valued(VarInt(7)));
		assert_eq!(c.values(4096).unwrap(), one);

		// 256 distinct values fit 8 bpe indirect, 257 do not and go direct.
		for (distinct, bpe) in [(2, 4), (256, 8), (257, 16), (4096, 16)] {
			// spread out, but below the block state count
			let values: Vec<i32> = (0..4096).map(|i| (i % distinct) * 8).collect();
			let c = PalletedContainer::from_values(&values, PaletteFormatType::BLOCKS).unwrap();
			assert_eq!(c.bits_per_entry, bpe, "{distinct} distinct values");
			assert_eq!(c.values(4096).unwrap(), values);

			// and it survives the wire format
			let bytes = to_bytes(&c);
			let back = PalletedContainer::mc_deserialize(&mut McDeserializer::new(&bytes), 4096, PaletteFormatType::BLOCKS).unwrap();
			assert_eq!(back.values(4096).unwrap(), values);
		}

		// 9 biomes need 4 bits, past the 3-bit indirect limit, so biomes go direct at 7 bits.
		let biomes: Vec<i32> = (0..64).map(|i| i % 9).collect();
		let c = PalletedContainer::from_values(&biomes, PaletteFormatType::BIOMES).unwrap();
		assert_eq!(c.bits_per_entry, 7);
		assert_eq!(c.values(64).unwrap(), biomes);

		// An ID that does not fit the direct bits must be an error, not a cut value.
		let mut too_big: Vec<i32> = (0..64).map(|i| i % 9).collect();
		too_big[0] = 128;
		assert!(PalletedContainer::from_values(&too_big, PaletteFormatType::BIOMES).is_err());
	}

	/// PackedXZ is x in the high nibble and z in the low nibble. Values over 15 would overwrite
	/// the other coordinate, so they must be refused.
	#[test]
	fn packed_xz() {
		let xz = PackedXZ::new(3, 12).unwrap();
		assert_eq!(to_bytes(&xz), vec![0x3C]);
		assert_eq!((xz.x(), xz.z()), (3, 12));

		let corner = PackedXZ::new(15, 15).unwrap();
		assert_eq!((corner.x(), corner.z()), (15, 15));

		assert!(PackedXZ::new(16, 0).is_err());
		assert!(PackedXZ::new(0, 16).is_err());
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

		assert!(light_array.get(4).is_ok());
	}
}
