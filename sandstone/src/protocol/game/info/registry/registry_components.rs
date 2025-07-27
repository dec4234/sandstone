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

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct ChatTypePart {
	translation_key: String,
	parameters: Vec<String>
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct ExitAction {
	width: i32,
	lavel: NbtTranslateColor
}

/// Monster spawn light level can either be a single integer value or a range. This handles this disambiguation basically
/// like a union.
#[derive(McDefault, Debug, Clone, PartialEq, McSerialize, McDeserialize)]
pub struct MonsterSpawnLightLevel {
	pub(crate) isRange: bool,
	pub(crate) level: Option<i32>,
	pub(crate) range: Option<MonsterSpawnLightLevelRange>,
}

impl From<MonsterSpawnLightLevel> for NbtCompound {
	fn from(value: MonsterSpawnLightLevel) -> Self {
		if value.isRange {
			value.range.unwrap().into()
		} else {
			panic!("Cannot convert MonsterSpawnLightLevel to NbtCompound without range");
		}
	}
}

impl From<MonsterSpawnLightLevel> for NbtTag {
	fn from(value: MonsterSpawnLightLevel) -> Self {
		if value.isRange {
			NbtTag::Compound(NbtCompound::from(value))
		} else {
			NbtTag::Int(value.level.unwrap_or(0))
		}
	}
}

impl From<NbtTag> for MonsterSpawnLightLevel {
	fn from(tag: NbtTag) -> Self {
		match tag {
			NbtTag::Int(level) => MonsterSpawnLightLevel {
				isRange: false,
				level: Some(level),
				range: None,
			},
			NbtTag::Compound(compound) => MonsterSpawnLightLevel {
				isRange: true,
				level: None,
				range: Some(MonsterSpawnLightLevelRange::from(compound)),
			},
			_ => panic!("Invalid NbtTag for MonsterSpawnLightLevel"),
		}
	}
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, McSerialize, McDeserialize)]
pub struct MonsterSpawnLightLevelRange {
	min_inclusive: i32,
	max_inclusive: i32,
	r#type: String,
}

// we need these custom impls because of course the field name is "type" which is a reserved keyword in Rust
impl From<NbtCompound> for MonsterSpawnLightLevelRange {
	fn from(compound: NbtCompound) -> Self {
		let min_inclusive: i32 = compound["min_inclusive"].clone().into();
		let max_inclusive: i32 = compound["max_inclusive"].clone().into();
		let r#type: String = compound["type"].clone().into();

		MonsterSpawnLightLevelRange {
			min_inclusive,
			max_inclusive,
			r#type,
		}
	}
}

impl Into<NbtCompound> for MonsterSpawnLightLevelRange {
	fn into(self) -> NbtCompound {
		let mut compound = NbtCompound::new_no_name();
		compound.add("min_inclusive".to_string(), self.min_inclusive);
		compound.add("max_inclusive".to_string(), self.max_inclusive);
		compound.add("type".to_string(), self.r#type);
		compound
	}
}

#[cfg(test)]
mod tests {
	use crate::protocol::game::info::registry::registry_components::{MonsterSpawnLightLevel, MonsterSpawnLightLevelRange};
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
	use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;

	/// Test that MonsterSpawnLightLevel can either be a struct or an integer
	#[test]
	fn test_monster_light_level_format() {
		let mut compound = NbtCompound::new_no_name();
		let light = MonsterSpawnLightLevel {
			isRange: true,
			level: None,
			range: Some(MonsterSpawnLightLevelRange {
				min_inclusive: 0,
				max_inclusive: 15,
				r#type: "minecraft:light_range".to_string(),
			}),
		};
		compound.add("light".to_string(), light);
		let light_num = MonsterSpawnLightLevel {
			isRange: false,
			level: Some(10),
			range: None,
		};
		compound.add("light_num".to_string(), light_num);

		println!("{}", serde_json::to_string(&compound).unwrap());

		let mut serializer = McSerializer::new();
		compound.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(serializer.as_bytes());
		let deserialized_compound = NbtCompound::mc_deserialize(&mut deserializer).unwrap();

		assert_eq!(compound, deserialized_compound);
	}
}