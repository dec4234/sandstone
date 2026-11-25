//! Change a [Chunk] to and from a `Chunk Data and Update Light` packet.
//!
//! Region files use names (block states, biomes, block entity types, heightmaps). The packet uses
//! numeric IDs. Block state and block entity IDs come from the generated vanilla table. Biome IDs
//! come from the biome registry that the server sent to the client, given in a [Dimension].

use crate::protocol::game::info::registry::vanilla_registries::SYNCED_REGISTRIES;
use crate::protocol::game::world::anvil::block_state::BlockState;
use crate::protocol::game::world::anvil::chunk::{BIOME_MIN_BITS, BLOCK_MIN_BITS, Chunk, DATA_VERSION, PalettedData, Section};
use crate::protocol::game::world::anvil::{AnvilError, AnvilResult};
use crate::protocol::game::world::block_state_table::BLOCK_ENTITY_TYPES;
use crate::protocol::game::world::chunk::{BlockEntity, ChunkByteData, ChunkData, ChunkSection, Heightmap, LightArray, LightData, PackedXZ, PaletteFormatType, PalletedContainer};
use crate::protocol::packets::ChunkDataUpdateLightPacket;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol_types::datatypes::nbt::{NbtCompound, NbtTag};
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::util::java::bitset::BitSet;
use std::collections::{BTreeMap, HashMap};

const BLOCKS_PER_SECTION: usize = 4096;
const BIOMES_PER_SECTION: usize = 64;
const LIGHT_BYTES: usize = 2048;

/// Heightmap names and their packet IDs. Vanilla sends only these three to the client.
const CLIENT_HEIGHTMAPS: [(&str, i32); 3] = [("WORLD_SURFACE", 1), ("MOTION_BLOCKING", 4), ("MOTION_BLOCKING_NO_LEAVES", 5)];

/// Block entity tags that the packet keeps in other fields.
const BLOCK_ENTITY_POSITION_TAGS: [&str; 5] = ["id", "x", "y", "z", "keepPacked"];

/// The height of a dimension and the biome registry that the client has. The conversion needs both:
/// the packet has one section for each section of the world height, and biomes by registry index.
#[derive(Debug, Clone, Copy)]
pub struct Dimension<'a> {
	/// The Y of the lowest section (`min_y / 16`). -4 in the vanilla overworld.
	pub min_section_y: i32,
	/// The number of sections (`height / 16`). 24 in the vanilla overworld.
	pub section_count: usize,
	/// The `minecraft:worldgen/biome` registry entries, in the order that the server sent them.
	pub biomes: &'a [&'a str],
}

impl Dimension<'static> {
	/// The vanilla overworld (-64 to 319) with the vanilla biome order.
	pub fn overworld() -> Self {
		Self {
			min_section_y: -4,
			section_count: 24,
			biomes: vanilla_biomes(),
		}
	}

	/// The vanilla nether and end (0 to 255) with the vanilla biome order.
	pub fn nether_or_end() -> Self {
		Self {
			min_section_y: 0,
			section_count: 16,
			biomes: vanilla_biomes(),
		}
	}
}

/// The vanilla biome registry order. This is the order that the login handler sends when the
/// client has the `minecraft:core` pack.
pub fn vanilla_biomes() -> &'static [&'static str] {
	SYNCED_REGISTRIES
		.iter()
		.find(|(registry, _)| *registry == "minecraft:worldgen/biome")
		.map(|(_, entries)| *entries)
		.expect("the vanilla registries include biomes")
}

impl Chunk {
	/// Make the packet that sends this chunk to a client.
	///
	/// - A section that the file does not have is sent as air with the plains biome, the same as vanilla loads it.
	/// - The light data is sent as the file has it. If `is_light_on` is not true, the light can be wrong.
	/// - Block entities are sent with all their saved data except the position and the type.
	///
	/// Only send chunks with `status` `minecraft:full`. Other chunks are not finished.
	pub fn to_packet(&self, dimension: &Dimension) -> AnvilResult<ChunkDataUpdateLightPacket> {
		let by_y: HashMap<i32, &Section> = self.sections.iter().map(|section| (section.y as i32, section)).collect();

		let mut sections = Vec::with_capacity(dimension.section_count);
		for y in dimension.min_section_y..dimension.min_section_y + dimension.section_count as i32 {
			sections.push(section_to_packet(by_y.get(&y).copied(), dimension)?);
		}

		let mut heightmaps = Vec::new();
		for (name, id) in CLIENT_HEIGHTMAPS {
			if let Some(data) = self.heightmaps.get(name) {
				heightmaps.push(Heightmap::new(VarInt(id), PrefixedArray::new(data.clone())));
			}
		}

		let block_entities = self.block_entities.iter().map(block_entity_to_packet).collect::<AnvilResult<_>>()?;

		Ok(ChunkDataUpdateLightPacket {
			x: self.x,
			z: self.z,
			data: ChunkData {
				heightmaps: PrefixedArray::new(heightmaps),
				data: ChunkByteData {
					data: sections,
				},
				block_entities: PrefixedArray::new(block_entities),
			},
			light: light_to_packet(&by_y, dimension),
		})
	}

	/// Make a chunk from a packet. The chunk has `status` `minecraft:full` and the data version of
	/// this library. Tags that the packet does not have (for example ticks and structures) are not
	/// set. Vanilla calculates missing heightmaps when it loads the chunk.
	pub fn from_packet(packet: &ChunkDataUpdateLightPacket, dimension: &Dimension) -> AnvilResult<Self> {
		let packet_sections = &packet.data.data.data;
		if packet_sections.len() != dimension.section_count {
			return Err(AnvilError::InvalidChunk(format!(
				"packet has {} sections, the dimension has {}",
				packet_sections.len(),
				dimension.section_count
			)));
		}

		// One more section below and above the world for light.
		let mut sections: Vec<Section> = (0..dimension.section_count + 2)
			.map(|i| Section {
				y: (dimension.min_section_y - 1 + i as i32) as i8,
				block_states: None,
				biomes: None,
				block_light: None,
				sky_light: None,
				other: NbtCompound::new_no_name(),
			})
			.collect();

		for (section, packet_section) in sections[1..].iter_mut().zip(packet_sections) {
			section_from_packet(section, packet_section, dimension)?;
		}
		light_from_packet(&mut sections, &packet.light)?;
		// Vanilla saves the sections below and above the world only if they have light.
		sections.retain(|s| s.block_states.is_some() || s.block_light.is_some() || s.sky_light.is_some());

		let mut heightmaps = BTreeMap::new();
		for heightmap in &packet.data.heightmaps.vec {
			let name = heightmap_name(heightmap.typ().0).ok_or_else(|| AnvilError::InvalidChunk(format!("unknown heightmap type {}", heightmap.typ().0)))?;
			heightmaps.insert(name.to_string(), heightmap.data().to_vec());
		}

		Ok(Self {
			data_version: DATA_VERSION,
			x: packet.x,
			z: packet.z,
			min_section_y: dimension.min_section_y,
			status: "minecraft:full".to_string(),
			sections,
			block_entities: packet
				.data
				.block_entities
				.vec
				.iter()
				.map(|entity| block_entity_from_packet(entity, packet.x, packet.z))
				.collect::<AnvilResult<_>>()?,
			heightmaps,
			is_light_on: Some(true),
			other: NbtCompound::new_no_name(),
		})
	}
}

fn section_to_packet(section: Option<&Section>, dimension: &Dimension) -> AnvilResult<ChunkSection> {
	let (block_ids, block_count, fluid_count) = match section.and_then(|s| s.block_states.as_ref()) {
		Some(states) => {
			let indices = states.indices(BLOCKS_PER_SECTION, BLOCK_MIN_BITS)?;
			let ids = states.palette.iter().map(BlockState::to_id).collect::<AnvilResult<Vec<_>>>()?;
			let counted: Vec<(i16, i16)> = states.palette.iter().map(|state| (!is_air(state) as i16, has_fluid(state) as i16)).collect();
			let mut block_count = 0;
			let mut fluid_count = 0;
			for &index in &indices {
				let (block, fluid) = counted[index as usize];
				block_count += block;
				fluid_count += fluid;
			}
			(indices.iter().map(|&i| ids[i as usize]).collect(), block_count, fluid_count)
		}
		None => (vec![BlockState::new("minecraft:air").to_id()?; BLOCKS_PER_SECTION], 0, 0),
	};

	let biome_ids = match section.and_then(|s| s.biomes.as_ref()) {
		Some(biomes) => {
			let indices = biomes.indices(BIOMES_PER_SECTION, BIOME_MIN_BITS)?;
			let ids = biomes.palette.iter().map(|name| biome_id(name, dimension)).collect::<AnvilResult<Vec<_>>>()?;
			indices.iter().map(|&i| ids[i as usize]).collect()
		}
		None => vec![biome_id("minecraft:plains", dimension)?; BIOMES_PER_SECTION],
	};

	Ok(ChunkSection {
		block_count,
		fluid_count,
		block_states: PalletedContainer::from_values(&block_ids, PaletteFormatType::BLOCKS)?,
		biomes: PalletedContainer::from_values(&biome_ids, PaletteFormatType::BIOMES)?,
	})
}

fn section_from_packet(section: &mut Section, packet: &ChunkSection, dimension: &Dimension) -> AnvilResult<()> {
	let block_ids = packet.block_states.values(BLOCKS_PER_SECTION)?;
	let (palette, indices) = palette_of(&block_ids, |id| BlockState::from_id(id).ok_or_else(|| AnvilError::UnknownBlock(format!("state ID {id}"))))?;
	section.block_states = Some(PalettedData::from_indices(palette, &indices, BLOCK_MIN_BITS));

	let biome_ids = packet.biomes.values(BIOMES_PER_SECTION)?;
	let (palette, indices) = palette_of(&biome_ids, |id| {
		dimension
			.biomes
			.get(id as usize)
			.map(|name| name.to_string())
			.ok_or_else(|| AnvilError::UnknownBiome(format!("biome ID {id}")))
	})?;
	section.biomes = Some(PalettedData::from_indices(palette, &indices, BIOME_MIN_BITS));
	Ok(())
}

/// The distinct values of `ids` (in order of first use) changed by `entry`, and the palette index of each ID.
fn palette_of<T>(ids: &[i32], entry: impl Fn(i32) -> AnvilResult<T>) -> AnvilResult<(Vec<T>, Vec<u16>)> {
	let mut seen: HashMap<i32, u16> = HashMap::new();
	let mut palette = Vec::new();
	let mut indices = Vec::with_capacity(ids.len());
	for &id in ids {
		let index = match seen.get(&id) {
			Some(&index) => index,
			None => {
				palette.push(entry(id)?);
				seen.insert(id, (palette.len() - 1) as u16);
				(palette.len() - 1) as u16
			}
		};
		indices.push(index);
	}
	Ok((palette, indices))
}

/// Air, cave air, and void air do not count as blocks in a section's block count.
fn is_air(state: &BlockState) -> bool {
	matches!(state.name.as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
}

/// The fluid count is the number of blocks with a fluid: water, lava, waterlogged blocks, and the
/// blocks that always hold water. The blocks report does not list fluids, so the last group is a
/// fixed list here.
fn has_fluid(state: &BlockState) -> bool {
	let always = matches!(
		state.name.as_str(),
		"minecraft:water" | "minecraft:lava" | "minecraft:bubble_column" | "minecraft:kelp" | "minecraft:kelp_plant" | "minecraft:seagrass" | "minecraft:tall_seagrass"
	);
	if always {
		return true;
	}
	match state.properties.get("waterlogged") {
		Some(value) => value == "true",
		// No property in the file means the default value. Look it up.
		None => BlockState::from_id(state.to_id().unwrap_or(0))
			.and_then(|s| s.properties.get("waterlogged").cloned())
			.is_some_and(|v| v == "true"),
	}
}

fn biome_id(name: &str, dimension: &Dimension) -> AnvilResult<i32> {
	dimension
		.biomes
		.iter()
		.position(|b| *b == name)
		.map(|i| i as i32)
		.ok_or_else(|| AnvilError::UnknownBiome(name.to_string()))
}

fn heightmap_name(id: i32) -> Option<&'static str> {
	// The order of Heightmap.Types in vanilla.
	["WORLD_SURFACE_WG", "WORLD_SURFACE", "OCEAN_FLOOR_WG", "OCEAN_FLOOR", "MOTION_BLOCKING", "MOTION_BLOCKING_NO_LEAVES"]
		.get(id as usize)
		.copied()
}

fn block_entity_to_packet(nbt: &NbtCompound) -> AnvilResult<BlockEntity> {
	let int = |name: &str| match nbt.get(name) {
		Some(NbtTag::Int(value)) => Ok(*value),
		other => Err(AnvilError::InvalidChunk(format!("block entity `{name}` is {other:?}, expected an int"))),
	};
	let id = nbt.get_string("id").ok_or_else(|| AnvilError::InvalidChunk("block entity without an `id`".to_string()))?;
	let typ = BLOCK_ENTITY_TYPES
		.iter()
		.position(|t| *t == id)
		.ok_or_else(|| AnvilError::UnknownBlock(format!("block entity type {id}")))?;

	let mut data = nbt.clone();
	data.root_name = None;
	for tag in BLOCK_ENTITY_POSITION_TAGS {
		data.remove(tag);
	}

	Ok(BlockEntity {
		packed_xz: PackedXZ::new((int("x")? & 15) as u8, (int("z")? & 15) as u8)?,
		y: int("y")? as i16,
		typ: VarInt(typ as i32),
		data,
	})
}

fn block_entity_from_packet(entity: &BlockEntity, chunk_x: i32, chunk_z: i32) -> AnvilResult<NbtCompound> {
	let id = BLOCK_ENTITY_TYPES
		.get(entity.typ.0 as usize)
		.ok_or_else(|| AnvilError::UnknownBlock(format!("block entity type ID {}", entity.typ.0)))?;
	let mut nbt = entity.data.clone();
	nbt.root_name = None;
	nbt.add("id", *id);
	nbt.add("x", chunk_x * 16 + entity.packed_xz.x() as i32);
	nbt.add("y", entity.y as i32);
	nbt.add("z", chunk_z * 16 + entity.packed_xz.z() as i32);
	nbt.add("keepPacked", NbtTag::Byte(0));
	Ok(nbt)
}

/// Light masks have one bit per section from one below the world to one above it. A set bit in a
/// mask means an array follows. A set bit in an empty mask means the section has no light (all
/// zero). No bit means the section has no light data, and the client works it out.
fn light_to_packet(by_y: &HashMap<i32, &Section>, dimension: &Dimension) -> LightData {
	let count = dimension.section_count + 2;
	let mut light = LightData {
		sky_light_mask: BitSet::new(count),
		block_light_mask: BitSet::new(count),
		empty_light_mask: BitSet::new(count),
		empty_block_light_mask: BitSet::new(count),
		sky_light: PrefixedArray::new(Vec::new()),
		block_light: PrefixedArray::new(Vec::new()),
	};

	for i in 0..count {
		let Some(section) = by_y.get(&(dimension.min_section_y - 1 + i as i32)) else {
			continue;
		};
		for (data, mask, empty_mask, arrays) in [
			(&section.sky_light, &mut light.sky_light_mask, &mut light.empty_light_mask, &mut light.sky_light),
			(&section.block_light, &mut light.block_light_mask, &mut light.empty_block_light_mask, &mut light.block_light),
		] {
			match data {
				Some(data) if data.iter().all(|&b| b == 0) => empty_mask.set_bit(i, true),
				Some(data) => {
					mask.set_bit(i, true);
					arrays.vec.push(LightArray {
						data: PrefixedArray::new(data.clone()),
					});
				}
				None => {}
			}
		}
	}

	light
}

/// `sections` has one entry per light section, from one below the world to one above it.
fn light_from_packet(sections: &mut [Section], light: &LightData) -> AnvilResult<()> {
	let mut sky = light.sky_light.vec.iter();
	let mut block = light.block_light.vec.iter();

	for (i, section) in sections.iter_mut().enumerate() {
		for (target, mask, empty_mask, arrays) in [
			(&mut section.sky_light, &light.sky_light_mask, &light.empty_light_mask, &mut sky),
			(&mut section.block_light, &light.block_light_mask, &light.empty_block_light_mask, &mut block),
		] {
			if mask.get_bit(i) {
				let array = arrays.next().ok_or_else(|| AnvilError::InvalidChunk("light mask has more bits than light arrays".to_string()))?;
				if array.data.vec.len() != LIGHT_BYTES {
					return Err(AnvilError::InvalidChunk(format!("light array has {} bytes, expected {LIGHT_BYTES}", array.data.vec.len())));
				}
				*target = Some(array.data.vec.clone());
			} else if empty_mask.get_bit(i) {
				*target = Some(vec![0; LIGHT_BYTES]);
			}
		}
	}

	if sky.next().is_some() || block.next().is_some() {
		return Err(AnvilError::InvalidChunk("more light arrays than light mask bits, or bits outside the world".to_string()));
	}
	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::protocol::game::world::block_states::{AIR, STONE, WATER};

	fn section(y: i8, palette: &[&str], indices: &[u16]) -> Section {
		Section {
			y,
			block_states: Some(PalettedData::from_indices(palette.iter().map(|n| BlockState::new(*n)).collect(), indices, BLOCK_MIN_BITS)),
			biomes: Some(PalettedData::from_indices(vec!["minecraft:desert".to_string()], &[0; 64], BIOME_MIN_BITS)),
			block_light: None,
			sky_light: None,
			other: NbtCompound::new_no_name(),
		}
	}

	fn chunk(sections: Vec<Section>) -> Chunk {
		Chunk {
			data_version: DATA_VERSION,
			x: -3,
			z: 7,
			min_section_y: -4,
			status: "minecraft:full".to_string(),
			sections,
			block_entities: vec![],
			heightmaps: BTreeMap::new(),
			is_light_on: Some(true),
			other: NbtCompound::new_no_name(),
		}
	}

	/// Block positions, block IDs, biome IDs, and counts must be right in the packet. The client
	/// uses the block count to skip empty sections, so a wrong count hides blocks.
	#[test]
	fn to_packet_puts_blocks_in_the_right_places() {
		// bottom layer stone, one water block above it, rest air
		let mut indices = vec![0u16; 4096];
		indices[..256].fill(1);
		indices[256 + 5] = 2;
		let packet = chunk(vec![section(-4, &["minecraft:air", "minecraft:stone", "minecraft:water"], &indices)])
			.to_packet(&Dimension::overworld())
			.unwrap();

		assert_eq!((packet.x, packet.z), (-3, 7));
		let sections = &packet.data.data.data;
		assert_eq!(sections.len(), 24);
		let bottom = &sections[0];
		assert_eq!((bottom.block_count, bottom.fluid_count), (257, 1));
		let ids = bottom.block_states.values(4096).unwrap();
		assert_eq!((ids[0], ids[255], ids[256], ids[261]), (STONE, STONE, AIR, WATER));

		let desert = vanilla_biomes().iter().position(|b| *b == "minecraft:desert").unwrap() as i32;
		assert_eq!(bottom.biomes, PalletedContainer::single_valued(VarInt(desert)));

		// A section that the file does not have is air with the plains biome.
		let plains = vanilla_biomes().iter().position(|b| *b == "minecraft:plains").unwrap() as i32;
		assert_eq!(sections[1].block_states, PalletedContainer::single_valued(VarInt(AIR)));
		assert_eq!(sections[1].biomes, PalletedContainer::single_valued(VarInt(plains)));
		assert_eq!(sections[1].block_count, 0);
	}

	/// The biome ID depends on the registry that the client got. With another order, the same name
	/// must give another ID.
	#[test]
	fn biome_ids_come_from_the_dimension() {
		let dimension = Dimension {
			min_section_y: -4,
			section_count: 24,
			biomes: &["minecraft:plains", "minecraft:desert"],
		};
		let packet = chunk(vec![section(-4, &["minecraft:air"], &[0; 4096])]).to_packet(&dimension).unwrap();
		assert_eq!(packet.data.data.data[0].biomes, PalletedContainer::single_valued(VarInt(1)));

		let missing = Dimension {
			biomes: &["minecraft:plains"],
			..dimension
		};
		assert!(matches!(chunk(vec![section(-4, &["minecraft:air"], &[0; 4096])]).to_packet(&missing), Err(AnvilError::UnknownBiome(_))));
	}

	#[test]
	fn fluid_count_includes_waterlogged_blocks() {
		let mut stairs = BlockState::new("minecraft:oak_stairs");
		assert!(!has_fluid(&stairs));
		stairs.properties.insert("waterlogged".to_string(), "true".to_string());
		assert!(has_fluid(&stairs));
		assert!(has_fluid(&BlockState::new("minecraft:kelp")));
		assert!(!has_fluid(&BlockState::new("minecraft:stone")));
	}

	/// Light masks start one section below the world. If the bit is off by one, every section is lit
	/// with its neighbor's light.
	#[test]
	fn light_bits_start_below_the_world() {
		let mut below = section(-5, &["minecraft:air"], &[0; 4096]);
		below.block_states = None;
		below.biomes = None;
		below.sky_light = Some(vec![0x11; 2048]);
		let mut bottom = section(-4, &["minecraft:air"], &[0; 4096]);
		bottom.block_light = Some(vec![0; 2048]);

		let packet = chunk(vec![below, bottom]).to_packet(&Dimension::overworld()).unwrap();
		let light = &packet.light;
		assert!(light.sky_light_mask.get_bit(0));
		assert!(!light.sky_light_mask.get_bit(1));
		assert!(light.empty_block_light_mask.get_bit(1));
		assert!(!light.block_light_mask.get_bit(1));
		assert_eq!(light.sky_light.vec.len(), 1);
		assert_eq!(light.block_light.vec.len(), 0);
	}

	/// A chunk made from its own packet must make the same packet. That is the property a server
	/// needs to save chunks that it sent and send them again later.
	#[test]
	fn packet_round_trip() {
		let mut indices = vec![0u16; 4096];
		for (i, index) in indices.iter_mut().enumerate() {
			*index = (i % 3) as u16;
		}
		let mut stairs = section(0, &["minecraft:air", "minecraft:stone", "minecraft:oak_stairs"], &indices);
		stairs.sky_light = Some(vec![0xF0; 2048]);
		stairs.block_light = Some(vec![0; 2048]);

		let mut original = chunk(vec![section(-4, &["minecraft:stone"], &[0; 4096]), stairs]);
		original.heightmaps.insert("MOTION_BLOCKING".to_string(), vec![1, 2, 3]);
		let mut chest = NbtCompound::new_no_name();
		chest.add("id", "minecraft:chest");
		chest.add("x", -3 * 16 + 4);
		chest.add("y", 70);
		chest.add("z", 7 * 16 + 15);
		chest.add("keepPacked", NbtTag::Byte(0));
		chest.add("CustomName", "box");
		original.block_entities.push(chest.clone());

		let packet = original.to_packet(&Dimension::overworld()).unwrap();
		let entity = &packet.data.block_entities.vec[0];
		assert_eq!((entity.packed_xz.x(), entity.packed_xz.z(), entity.y), (4, 15, 70));
		assert_eq!(entity.data.get_string("CustomName").as_deref(), Some("box"));
		assert!(entity.data.get("x").is_none(), "position tags are packet fields, not NBT");

		let back = Chunk::from_packet(&packet, &Dimension::overworld()).unwrap();
		assert_eq!(back.block_entities, vec![chest]);
		assert_eq!(back.heightmaps, original.heightmaps);
		assert_eq!(back.to_packet(&Dimension::overworld()).unwrap(), packet);

		// The section with stairs keeps its blocks.
		let section = back.sections.iter().find(|s| s.y == 0).unwrap();
		let states = section.block_states.as_ref().unwrap();
		let ids: Vec<i32> = states.indices(4096, BLOCK_MIN_BITS).unwrap().iter().map(|&i| states.palette[i as usize].to_id().unwrap()).collect();
		assert_eq!(ids[2], BlockState::new("minecraft:oak_stairs").to_id().unwrap());
		assert_eq!(section.block_light, Some(vec![0; 2048]));
	}

	#[test]
	fn from_packet_refuses_a_wrong_height() {
		let packet = chunk(vec![]).to_packet(&Dimension::overworld()).unwrap();
		assert!(Chunk::from_packet(&packet, &Dimension::nether_or_end()).is_err());
	}
}
