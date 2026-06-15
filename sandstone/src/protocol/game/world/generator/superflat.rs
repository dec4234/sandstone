//! A minimal superflat world generator. Produces a single chunk (every chunk in a superflat world
//! is identical) ready to be sent in a `Chunk Data and Update Light` packet.

use crate::protocol::game::world::chunk::{
	ChunkByteData, ChunkData, ChunkSection, LightArray, LightData, PaletteFormatType,
	PalletedContainer,
};
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::util::java::bitset::BitSet;
use mc_data::Block;

// Global block-state IDs for protocol 774 (1.21.6). See the blocks report / minecraft-data.
const AIR: i32 = Block::Air::get_max_id() as i32;
const GRASS_BLOCK: i32 = Block::GrassBlock::get_max_id() as i32; // grass_block, snowy=false
const DIRT: i32 = Block::Dirt::get_max_id() as i32;
const BEDROCK: i32 = Block::Bedrock::get_max_id() as i32;

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
		data: ChunkByteData { data: sections },
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
		block_states: PalletedContainer::indirect(palette, &indices, PaletteFormatType::BLOCKS)
			.unwrap(),
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
