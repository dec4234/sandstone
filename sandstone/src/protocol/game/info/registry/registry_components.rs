//! Complex and nested registry components. Like biome effects, music data, etc.

use crate::protocol::game::info::registry::McDeserialize;
use crate::protocol::game::info::registry::McDeserializer;
use crate::protocol::game::info::registry::McSerialize;
use crate::protocol::game::info::registry::McSerializer;
use crate::protocol::game::info::registry::NbtCompound;
use crate::protocol::game::info::registry::SerializingErr;
use crate::protocol::game::info::registry::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::nbt::nbt::NbtTag;
use sandstone_derive::{AsNbt, FromNbt, McDefault, McDeserialize};
use serde::{Deserialize, Serialize};

/// Used for some sections of registry components such as painting_variant
#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct NbtTranslateColor {
	pub color: Option<String>,
	pub translate: String,
}

/// Used for "minecraft:worldgen/biome" registry component
#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeEffects {
	fog_color: i32,
	foliage_color: i32,
	grass_color: i32,
	mood_sound: BiomeMood,
	music: Vec<BiomeMusicData>,
	music_volume: f32,
	sky_color: i32,
	water_color: i32,
	water_fog_color: i32,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeMood {
	block_search_extent: i32,
	offset: f64,
	sound: String,
	tick_delay: i32,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeMusicData {
	data: BiomeMusic,
	weight: i32
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeMusic {
	max_delay: i32,
	min_delay: i32,
	replace_current_music: bool,
	sound: String,
}

