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
	pub translate: String,
	pub color: Option<String>,
}

/// Used for "minecraft:worldgen/biome" registry component
#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeEffects {
	fog_color: i32,
	foliage_color: i32,
	grass_color: i32,
	mood_sound: BiomeMood,
	//music: Vec<BiomeMusic>,
	music_volume: f32,
	sky_color: i32,
	water_color: i32,
	water_fog_color: i32,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeMood {
	offset: f64,
	block_search_extent: i32,
	tick_delay: i32,
	sound: String
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeMusic {

}

