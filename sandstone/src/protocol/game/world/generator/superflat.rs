//! A minimal superflat world generator. Produces a single chunk (every chunk in a superflat world
//! is identical) ready to be sent in a `Chunk Data and Update Light` packet.

use crate::protocol::game::world::chunk::{ChunkByteData, ChunkData, ChunkSection, LightArray, LightData, PaletteFormatType, PalletedContainer};
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::util::java::bitset::BitSet;
use mc_data::Block;

const AIR: i32 = Block::Air::get_max_state_id() as i32;
const GRASS_BLOCK: i32 = Block::GrassBlock::get_max_state_id() as i32;
const DIRT: i32 = Block::Dirt::get_max_state_id() as i32;
const BEDROCK: i32 = Block::Bedrock::get_max_state_id() as i32;

// biome ID that matches the index in the registry for biomes
const PLAINS_BIOME: i32 = 0;

// Overworld is 384 blocks tall (registry height 384, min_y -64) -> 24 sections.
const SECTION_COUNT: usize = 24;
const LIGHT_SECTION_COUNT: usize = SECTION_COUNT + 2;

/// Generate the chunk used for every column of a superflat world: bedrock, two dirt, then a grass
/// block on top (world y -64..-61), air above. Returns the chunk block data and a fully sky-lit
/// `LightData` so the world is visible.
pub fn superflat_chunk() -> (ChunkData, LightData) {
	let mut sections = Vec::with_capacity(SECTION_COUNT);
	sections.push(bottom_section());
	for _ in 1..SECTION_COUNT {
		sections.push(air_section());
	}

	let data = ChunkData {
		heightmaps: PrefixedArray::new(vec![]),
		data: ChunkByteData {
			data: sections,
		},
		block_entities: PrefixedArray::new(vec![]),
	};

	(data, lit_light_data())
}

/// The bottom section, containing the four solid layers. Entries are laid out YZX:
/// index = (y << 8) | (z << 4) | x, with y/z/x each 0..15.
fn bottom_section() -> ChunkSection {
	let palette = vec![VarInt(AIR), VarInt(BEDROCK), VarInt(DIRT), VarInt(GRASS_BLOCK)];
	let mut indices = vec![0u16; 4096];

	for y in 0..16usize {
		let block: u16 = match y {
			0 => 1,     // bedrock
			1 | 2 => 2, // dirt
			3 => 3,     // grass_block
			_ => 0,     // air
		};
		if block == 0 {
			continue;
		}
		for z in 0..16usize {
			for x in 0..16usize {
				indices[(y << 8) | (z << 4) | x] = block;
			}
		}
	}

	// bedrock + 2 dirt + grass = 4 full layers of 16x16 non-air blocks
	let block_count = 16 * 16 * 4;

	ChunkSection {
		block_count: block_count as i16,
		block_states: PalletedContainer::indirect(palette, &indices, PaletteFormatType::BLOCKS).unwrap(),
		biomes: PalletedContainer::single_valued(VarInt(PLAINS_BIOME)),
	}
}

/// An empty (all-air) section.
fn air_section() -> ChunkSection {
	ChunkSection {
		block_count: 0,
		block_states: PalletedContainer::single_valued(VarInt(AIR)),
		biomes: PalletedContainer::single_valued(VarInt(PLAINS_BIOME)),
	}
}

/// Full sky light for every section (all nibbles 0x0F), no block light.
fn lit_light_data() -> LightData {
	let mut sky_mask = BitSet::new(LIGHT_SECTION_COUNT);
	let mut empty_block_mask = BitSet::new(LIGHT_SECTION_COUNT);
	for i in 0..LIGHT_SECTION_COUNT {
		sky_mask.set_bit(i, true);
		empty_block_mask.set_bit(i, true);
	}

	let sky_light: Vec<LightArray> = (0..LIGHT_SECTION_COUNT)
		.map(|_| LightArray {
			data: PrefixedArray::new(vec![0xFF; 2048]),
		})
		.collect();

	LightData {
		sky_light_mask: sky_mask,
		block_light_mask: BitSet::new(LIGHT_SECTION_COUNT),
		empty_light_mask: BitSet::new(LIGHT_SECTION_COUNT),
		empty_block_light_mask: empty_block_mask,
		sky_light: PrefixedArray::new(sky_light),
		block_light: PrefixedArray::new(vec![]),
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::protocol::game::world::chunk::PalleteFormat;
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};

	/// Look up the block state at a YZX index in a section.
	fn block_at(section: &ChunkSection, y: usize, z: usize, x: usize) -> i32 {
		let c = &section.block_states;
		match &c.palette {
			PalleteFormat::SingleValued(v) => v.0,
			PalleteFormat::Indirect(f) => {
				let i = (y << 8) | (z << 4) | x;
				let per_long = 64 / c.bits_per_entry as usize;
				let palette_index = c.data[i / per_long].get_entry((i % per_long) as u8);
				f.palette.vec[palette_index as usize].0
			}
			PalleteFormat::Direct => panic!("superflat does not use the direct format"),
		}
	}

	#[test]
	fn one_section_per_16_blocks_of_overworld_height() {
		// The client reads exactly height / 16 sections. 384 / 16 = 24.
		let (chunk, _) = superflat_chunk();
		assert_eq!(chunk.data.data.len(), 24);
	}

	#[test]
	fn bottom_layers_are_bedrock_dirt_dirt_grass() {
		let (chunk, _) = superflat_chunk();
		let bottom = &chunk.data.data[0];
		for (x, z) in [(0, 0), (15, 15), (7, 3)] {
			assert_eq!(block_at(bottom, 0, z, x), BEDROCK);
			assert_eq!(block_at(bottom, 1, z, x), DIRT);
			assert_eq!(block_at(bottom, 2, z, x), DIRT);
			assert_eq!(block_at(bottom, 3, z, x), GRASS_BLOCK);
			assert_eq!(block_at(bottom, 4, z, x), AIR);
			assert_eq!(block_at(bottom, 15, z, x), AIR);
		}
		for section in &chunk.data.data[1..] {
			assert_eq!(block_at(section, 0, 0, 0), AIR);
		}
	}

	#[test]
	fn block_count_matches_non_air_blocks() {
		// The client uses Block Count to skip rendering empty sections, so it must be the real count.
		let (chunk, _) = superflat_chunk();
		for section in &chunk.data.data {
			let mut non_air = 0;
			for y in 0..16 {
				for z in 0..16 {
					for x in 0..16 {
						if block_at(section, y, z, x) != AIR {
							non_air += 1;
						}
					}
				}
			}
			assert_eq!(section.block_count as i32, non_air);
		}
	}

	#[test]
	fn light_arrays_match_masks() {
		// Wiki (Light Data): one 2048-byte array per bit set in each mask, in order. The overworld
		// has 24 sections plus one above and one below.
		let (_, light) = superflat_chunk();
		let sky_bits = (0..LIGHT_SECTION_COUNT).filter(|&i| light.sky_light_mask.get_bit(i)).count();
		let block_bits = (0..LIGHT_SECTION_COUNT).filter(|&i| light.block_light_mask.get_bit(i)).count();
		assert_eq!(sky_bits, 26);
		assert_eq!(light.sky_light.vec.len(), sky_bits);
		assert_eq!(light.block_light.vec.len(), block_bits);
		for arr in &light.sky_light.vec {
			assert_eq!(arr.data.vec.len(), 2048);
			assert_eq!(arr.get(0).unwrap(), 15);
			assert_eq!(arr.get(4095).unwrap(), 15);
		}
		// A section can not be both "has light data" and "empty".
		for i in 0..LIGHT_SECTION_COUNT {
			assert!(!(light.sky_light_mask.get_bit(i) && light.empty_light_mask.get_bit(i)));
		}
	}

	#[test]
	fn chunk_and_light_round_trip() {
		// A full encode then decode proves that the client can parse the sections with the
		// palette sizes we picked.
		let (chunk, light) = superflat_chunk();

		let mut s = McSerializer::new();
		chunk.mc_serialize(&mut s).unwrap();
		let mut d = McDeserializer::new(s.as_bytes());
		assert_eq!(ChunkData::mc_deserialize(&mut d).unwrap(), chunk);
		assert!(d.is_at_end());

		let mut s = McSerializer::new();
		light.mc_serialize(&mut s).unwrap();
		let mut d = McDeserializer::new(s.as_bytes());
		assert_eq!(LightData::mc_deserialize(&mut d).unwrap(), light);
		assert!(d.is_at_end());
	}
}
