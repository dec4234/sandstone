//! The typed form of the NBT of one chunk in a region file.
//!
//! <https://minecraft.wiki/w/Chunk_format>

use crate::protocol::game::world::anvil::block_state::BlockState;
use crate::protocol::game::world::anvil::{AnvilError, AnvilResult};
use crate::protocol_types::datatypes::nbt::{NbtByteArray, NbtCompound, NbtList, NbtLongArray, NbtTag};
use std::collections::BTreeMap;

/// The data version of the 26.3 vanilla server (`world_version` in its `version.json`). The block
/// state table and the vanilla biome list come from the same version.
pub const DATA_VERSION: i32 = 5023;

/// One chunk (a 16 block wide column) as a region file keeps it.
///
/// [Chunk::from_nbt] takes the tags that this type knows and keeps every other tag in `other`.
/// [Chunk::to_nbt] writes them all back, so a read and a write do not lose data.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
	/// `DataVersion`: the version of the game that saved the chunk.
	pub data_version: i32,
	/// `xPos`: the absolute chunk X coordinate.
	pub x: i32,
	/// `zPos`: the absolute chunk Z coordinate.
	pub z: i32,
	/// `yPos`: the Y of the lowest section of the world.
	pub min_section_y: i32,
	/// `Status`: how far world generation got. Only `minecraft:full` chunks are ready to send to a client.
	pub status: String,
	/// `sections`: the sections in the file. A section can have only light data.
	pub sections: Vec<Section>,
	/// `block_entities`: each has `id`, `x`, `y`, `z` (absolute block coordinates) and its own data.
	pub block_entities: Vec<NbtCompound>,
	/// `Heightmaps`: name (for example `MOTION_BLOCKING`) to packed heights.
	pub heightmaps: BTreeMap<String, Vec<i64>>,
	/// `isLightOn`: true if the light data is complete. Vanilla calculates the light again if it is false.
	pub is_light_on: Option<bool>,
	/// Every other tag of the chunk (for example ticks, structures, and `InhabitedTime`).
	pub other: NbtCompound,
}

/// One 16x16x16 section of a chunk.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
	/// `Y`: the section Y coordinate (block Y >> 4).
	pub y: i8,
	/// `block_states`: 4096 entries in YZX order (`y * 256 + z * 16 + x`).
	pub block_states: Option<PalettedData<BlockState>>,
	/// `biomes`: 64 entries for 4x4x4 cells, in YZX order. Each is a biome name, for example `minecraft:plains`.
	pub biomes: Option<PalettedData<String>>,
	/// `BlockLight`: 2048 bytes, one 4 bit value per block. The low nibble is the even index.
	pub block_light: Option<Vec<u8>>,
	/// `SkyLight`: same layout as `block_light`.
	pub sky_light: Option<Vec<u8>>,
	/// Every other tag of the section.
	pub other: NbtCompound,
}

/// A palette and the packed palette index of each entry. Entries do not cross a long. If the
/// palette has one value, `data` is empty.
#[derive(Debug, Clone, PartialEq)]
pub struct PalettedData<T> {
	pub palette: Vec<T>,
	pub data: Vec<i64>,
}

/// The fewest bits for block state palette indices in a region file.
pub const BLOCK_MIN_BITS: u32 = 4;
/// Biome palette indices have no lower limit.
pub const BIOME_MIN_BITS: u32 = 1;

impl<T> PalettedData<T> {
	/// Pack `indices` into palette `palette`. `min_bits` is [BLOCK_MIN_BITS] for block states and
	/// [BIOME_MIN_BITS] for biomes.
	pub fn from_indices(palette: Vec<T>, indices: &[u16], min_bits: u32) -> Self {
		let bits = bits_for(palette.len(), min_bits);
		if bits == 0 {
			return Self {
				palette,
				data: Vec::new(),
			};
		}

		let per_long = (64 / bits) as usize;
		let mut data = vec![0i64; indices.len().div_ceil(per_long)];
		for (i, &index) in indices.iter().enumerate() {
			data[i / per_long] |= (index as i64) << ((i % per_long) as u32 * bits);
		}
		Self {
			palette,
			data,
		}
	}

	/// The palette index of each of the `count` entries.
	pub fn indices(&self, count: usize, min_bits: u32) -> AnvilResult<Vec<u16>> {
		let bits = bits_for(self.palette.len(), min_bits);
		if self.palette.is_empty() {
			return Err(AnvilError::InvalidChunk("empty palette".to_string()));
		}
		if bits == 0 {
			return Ok(vec![0; count]);
		}

		let per_long = (64 / bits) as usize;
		if self.data.len() != count.div_ceil(per_long) {
			return Err(AnvilError::InvalidChunk(format!(
				"{} longs for {count} entries at {bits} bits, expected {}",
				self.data.len(),
				count.div_ceil(per_long)
			)));
		}

		let mask = (1u64 << bits) - 1;
		(0..count)
			.map(|i| {
				let index = ((self.data[i / per_long] as u64 >> ((i % per_long) as u32 * bits)) & mask) as u16;
				if index as usize >= self.palette.len() {
					return Err(AnvilError::InvalidChunk(format!("palette index {index} is out of bounds for {} entries", self.palette.len())));
				}
				Ok(index)
			})
			.collect()
	}
}

/// `max(min_bits, ceil(log2(len)))`, or 0 if the palette has one value.
fn bits_for(len: usize, min_bits: u32) -> u32 {
	if len <= 1 {
		return 0;
	}
	(usize::BITS - (len - 1).leading_zeros()).max(min_bits)
}

impl Chunk {
	/// Read a chunk from the NBT of a region file.
	pub fn from_nbt(mut nbt: NbtCompound) -> AnvilResult<Self> {
		let chunk = Self {
			data_version: take_int(&mut nbt, "DataVersion")?,
			x: take_int(&mut nbt, "xPos")?,
			z: take_int(&mut nbt, "zPos")?,
			min_section_y: take_int(&mut nbt, "yPos")?,
			status: take_string(&mut nbt, "Status")?,
			sections: take_list(&mut nbt, "sections")?
				.into_iter()
				.map(|tag| Section::from_nbt(compound(tag, "sections")?))
				.collect::<AnvilResult<_>>()?,
			block_entities: take_list(&mut nbt, "block_entities")?
				.into_iter()
				.map(|tag| compound(tag, "block_entities"))
				.collect::<AnvilResult<_>>()?,
			heightmaps: match nbt.map.remove("Heightmaps") {
				Some(tag) => compound(tag, "Heightmaps")?
					.map
					.into_iter()
					.map(|(name, tag)| Ok((name, long_array(tag, "Heightmaps")?)))
					.collect::<AnvilResult<_>>()?,
				None => BTreeMap::new(),
			},
			is_light_on: take_bool(&mut nbt, "isLightOn")?,
			other: NbtCompound::new_no_name(),
		};

		nbt.root_name = None;
		Ok(Self {
			other: nbt,
			..chunk
		})
	}

	/// Write the chunk as the NBT of a region file.
	pub fn to_nbt(&self) -> NbtCompound {
		let mut nbt = self.other.clone();
		nbt.root_name = Some(String::new());
		nbt.add("DataVersion", self.data_version);
		nbt.add("xPos", self.x);
		nbt.add("zPos", self.z);
		nbt.add("yPos", self.min_section_y);
		nbt.add("Status", self.status.as_str());
		nbt.add("sections", list(self.sections.iter().map(|section| NbtTag::Compound(section.to_nbt())).collect()));
		nbt.add("block_entities", list(self.block_entities.iter().cloned().map(NbtTag::Compound).collect()));
		// Vanilla always writes `sections`, `block_entities`, and `Heightmaps`, also when they are empty.
		let mut heightmaps = NbtCompound::new_no_name();
		for (name, data) in &self.heightmaps {
			heightmaps.add(name.as_str(), NbtTag::LongArray(NbtLongArray::new(data.clone())));
		}
		nbt.add("Heightmaps", heightmaps);
		if let Some(is_light_on) = self.is_light_on {
			nbt.add("isLightOn", NbtTag::Byte(is_light_on as i8));
		}
		nbt
	}
}

impl Section {
	fn from_nbt(mut nbt: NbtCompound) -> AnvilResult<Self> {
		let y = match nbt.map.remove("Y") {
			Some(NbtTag::Byte(y)) => y,
			other => return Err(AnvilError::InvalidChunk(format!("section `Y` is {other:?}, expected a byte"))),
		};

		let block_states = match nbt.map.remove("block_states") {
			Some(tag) => Some(paletted(compound(tag, "block_states")?, block_state_from_nbt)?),
			None => None,
		};
		let biomes = match nbt.map.remove("biomes") {
			Some(tag) => Some(paletted(compound(tag, "biomes")?, |tag| match tag {
				NbtTag::String(name) => Ok(name),
				other => Err(AnvilError::InvalidChunk(format!("biome palette entry is {other:?}, expected a string"))),
			})?),
			None => None,
		};

		Ok(Self {
			y,
			block_states,
			biomes,
			block_light: take_light(&mut nbt, "BlockLight")?,
			sky_light: take_light(&mut nbt, "SkyLight")?,
			other: nbt,
		})
	}

	fn to_nbt(&self) -> NbtCompound {
		let mut nbt = self.other.clone();
		nbt.add("Y", NbtTag::Byte(self.y));
		if let Some(block_states) = &self.block_states {
			nbt.add("block_states", paletted_to_nbt(block_states, block_state_to_nbt));
		}
		if let Some(biomes) = &self.biomes {
			nbt.add("biomes", paletted_to_nbt(biomes, |name| NbtTag::String(name.clone())));
		}
		for (name, light) in [("BlockLight", &self.block_light), ("SkyLight", &self.sky_light)] {
			if let Some(light) = light {
				nbt.add(name, NbtTag::ByteArray(NbtByteArray::new(light.iter().map(|&b| b as i8).collect())));
			}
		}
		nbt
	}
}

/// A block state palette entry is a string (the default state of that block) or a compound with
/// `id` and `properties`. A list with both kinds wraps the strings as `{"": "minecraft:air"}`.
fn block_state_from_nbt(tag: NbtTag) -> AnvilResult<BlockState> {
	match tag.unwrap_list_item() {
		NbtTag::String(name) => Ok(BlockState::new(name)),
		NbtTag::Compound(mut compound) => {
			let name = take_string(&mut compound, "id")?;
			let mut properties = BTreeMap::new();
			if let Some(tag) = compound.map.remove("properties") {
				for (key, value) in self::compound(tag, "properties")?.map {
					match value {
						NbtTag::String(value) => properties.insert(key, value),
						other => return Err(AnvilError::InvalidChunk(format!("{name}: property `{key}` is {other:?}, expected a string"))),
					};
				}
			}
			Ok(BlockState {
				name,
				properties,
			})
		}
		other => Err(AnvilError::InvalidChunk(format!("block state palette entry is {other:?}"))),
	}
}

/// The default state is written as its name, like vanilla does. Other states are a compound.
fn block_state_to_nbt(state: &BlockState) -> NbtTag {
	if state.properties.is_empty() || state.is_default().unwrap_or(false) {
		return NbtTag::String(state.name.clone());
	}
	let mut properties = NbtCompound::new_no_name();
	for (key, value) in &state.properties {
		properties.add(key.as_str(), value.as_str());
	}
	let mut compound = NbtCompound::new_no_name();
	compound.add("id", state.name.as_str());
	compound.add("properties", properties);
	NbtTag::Compound(compound)
}

fn paletted<T>(mut nbt: NbtCompound, entry: impl Fn(NbtTag) -> AnvilResult<T>) -> AnvilResult<PalettedData<T>> {
	Ok(PalettedData {
		palette: take_list(&mut nbt, "palette")?.into_iter().map(entry).collect::<AnvilResult<_>>()?,
		data: match nbt.map.remove("data") {
			Some(tag) => long_array(tag, "data")?,
			None => Vec::new(),
		},
	})
}

fn paletted_to_nbt<T>(data: &PalettedData<T>, entry: impl Fn(&T) -> NbtTag) -> NbtCompound {
	let mut nbt = NbtCompound::new_no_name();
	let palette = data.palette.iter().map(entry).collect();
	nbt.add("palette", NbtList::from_vec_wrapping(palette).expect("palette entries are strings or compounds"));
	if !data.data.is_empty() {
		nbt.add("data", NbtTag::LongArray(NbtLongArray::new(data.data.clone())));
	}
	nbt
}

fn list(tags: Vec<NbtTag>) -> NbtList {
	NbtList::from_vec(tags).expect("all items have the same type")
}

fn missing(name: &str) -> AnvilError {
	AnvilError::InvalidChunk(format!("missing `{name}`"))
}

fn wrong_type(name: &str, tag: &NbtTag) -> AnvilError {
	AnvilError::InvalidChunk(format!("`{name}` has the wrong type: {tag:?}"))
}

fn take_int(nbt: &mut NbtCompound, name: &str) -> AnvilResult<i32> {
	match nbt.map.remove(name) {
		Some(NbtTag::Int(value)) => Ok(value),
		Some(tag) => Err(wrong_type(name, &tag)),
		None => Err(missing(name)),
	}
}

fn take_string(nbt: &mut NbtCompound, name: &str) -> AnvilResult<String> {
	match nbt.map.remove(name) {
		Some(NbtTag::String(value)) => Ok(value),
		Some(tag) => Err(wrong_type(name, &tag)),
		None => Err(missing(name)),
	}
}

fn take_bool(nbt: &mut NbtCompound, name: &str) -> AnvilResult<Option<bool>> {
	match nbt.map.remove(name) {
		Some(NbtTag::Byte(value)) => Ok(Some(value != 0)),
		Some(tag) => Err(wrong_type(name, &tag)),
		None => Ok(None),
	}
}

/// A missing list is the same as an empty list.
fn take_list(nbt: &mut NbtCompound, name: &str) -> AnvilResult<Vec<NbtTag>> {
	match nbt.map.remove(name) {
		Some(NbtTag::List(list)) => Ok(list.list),
		Some(tag) => Err(wrong_type(name, &tag)),
		None => Ok(Vec::new()),
	}
}

fn take_light(nbt: &mut NbtCompound, name: &str) -> AnvilResult<Option<Vec<u8>>> {
	match nbt.map.remove(name) {
		Some(NbtTag::ByteArray(array)) if array.list.len() == 2048 => Ok(Some(array.list.into_iter().map(|b| b as u8).collect())),
		Some(tag) => Err(wrong_type(name, &tag)),
		None => Ok(None),
	}
}

fn compound(tag: NbtTag, name: &str) -> AnvilResult<NbtCompound> {
	match tag {
		NbtTag::Compound(compound) => Ok(compound),
		tag => Err(wrong_type(name, &tag)),
	}
}

fn long_array(tag: NbtTag, name: &str) -> AnvilResult<Vec<i64>> {
	match tag {
		NbtTag::LongArray(array) => Ok(array.list),
		tag => Err(wrong_type(name, &tag)),
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn bits_follow_the_region_rules() {
		// Block states: at least 4 bits. Biomes: no lower limit. One value: no data.
		assert_eq!(bits_for(1, BLOCK_MIN_BITS), 0);
		assert_eq!(bits_for(2, BLOCK_MIN_BITS), 4);
		assert_eq!(bits_for(17, BLOCK_MIN_BITS), 5);
		assert_eq!(bits_for(1, BIOME_MIN_BITS), 0);
		assert_eq!(bits_for(2, BIOME_MIN_BITS), 1);
		assert_eq!(bits_for(5, BIOME_MIN_BITS), 3);
		assert_eq!(bits_for(64, BIOME_MIN_BITS), 6);
	}

	/// 4096 entries at 5 bits is 12 per long (4 bits of padding), so 342 longs. Packing across longs
	/// would give 320 longs, and vanilla would read every block after the first long wrong.
	#[test]
	fn indices_do_not_cross_longs() {
		let palette: Vec<u8> = (0..17).collect();
		let indices: Vec<u16> = (0..4096).map(|i| (i % 17) as u16).collect();
		let data = PalettedData::from_indices(palette, &indices, BLOCK_MIN_BITS);
		assert_eq!(data.data.len(), 342);
		// entry 12 is the first entry of the second long
		assert_eq!(data.data[1] & 0x1F, 12);
		assert_eq!(data.indices(4096, BLOCK_MIN_BITS).unwrap(), indices);
	}

	#[test]
	fn bad_data_is_an_error() {
		let short = PalettedData {
			palette: vec![1, 2],
			data: vec![0; 10],
		};
		assert!(short.indices(4096, BLOCK_MIN_BITS).is_err());

		// index 3 with a palette of 3 entries
		let out_of_bounds = PalettedData {
			palette: vec![1, 2, 3],
			data: vec![3; 2],
		};
		assert!(out_of_bounds.indices(64, BIOME_MIN_BITS).is_err());
	}

	/// The palette is a mixed list in 26.3. Strings must stay strings and states with properties
	/// must stay compounds, or vanilla reads a different block.
	#[test]
	fn mixed_palette_round_trips() {
		let mut stairs = BlockState::new("minecraft:oak_stairs");
		stairs.properties.insert("facing".to_string(), "east".to_string());
		let section = Section {
			y: -4,
			block_states: Some(PalettedData::from_indices(vec![BlockState::new("minecraft:air"), stairs.clone()], &[1; 4096], BLOCK_MIN_BITS)),
			biomes: Some(PalettedData::from_indices(vec!["minecraft:plains".to_string()], &[0; 64], BIOME_MIN_BITS)),
			block_light: None,
			sky_light: Some(vec![0xFF; 2048]),
			other: NbtCompound::new_no_name(),
		};
		let nbt = section.to_nbt();

		let palette = &nbt["block_states"];
		let NbtTag::Compound(block_states) = palette else { panic!() };
		let NbtTag::List(palette) = &block_states["palette"] else { panic!() };
		assert_eq!(palette.list[0].clone().unwrap_list_item(), NbtTag::String("minecraft:air".to_string()));
		assert!(matches!(&palette.list[1], NbtTag::Compound(c) if c.get_string("id").as_deref() == Some("minecraft:oak_stairs")));

		assert_eq!(Section::from_nbt(nbt).unwrap(), section);
	}

	/// A state with all default properties is written as a string, the same as vanilla.
	#[test]
	fn default_state_is_written_as_a_string() {
		let mut grass = BlockState::new("minecraft:grass_block");
		grass.properties.insert("snowy".to_string(), "false".to_string());
		assert_eq!(block_state_to_nbt(&grass), NbtTag::String("minecraft:grass_block".to_string()));
	}
}
