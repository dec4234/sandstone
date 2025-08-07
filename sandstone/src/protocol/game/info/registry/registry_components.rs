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
use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
use sandstone_derive::nbt;
use sandstone_derive::{AsNbt, FromNbt, McDefault, McDeserialize};
use serde::{Deserialize, Serialize};

/// Used for some sections of registry components such as painting_variant
#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct NbtTranslateColor {
	pub color: Option<String>,
	pub translate: String,
}

impl From<NbtTag> for Option<NbtTranslateColor> {
	fn from(tag: NbtTag) -> Self {
		match NbtTranslateColor::try_from(tag) {
			Ok(value) => Some(value),
			Err(_) => None,
		}
	}
}

/// Used for "minecraft:worldgen/biome" registry component
#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeEffects {
	pub fog_color: i32,
	pub foliage_color: i32,
	pub grass_color: i32,
	pub mood_sound: BiomeMood,
	pub music: Vec<BiomeMusicData>,
	pub music_volume: f32,
	pub sky_color: i32,
	pub water_color: i32,
	pub water_fog_color: i32,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeMood {
	pub block_search_extent: i32,
	pub offset: f64,
	pub sound: String,
	pub tick_delay: i32,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeMusicData {
	pub data: BiomeMusic,
	pub weight: i32
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BiomeMusic {
	pub max_delay: i32,
	pub min_delay: i32,
	pub replace_current_music: bool,
	pub sound: String,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct ChatTypePart {
	pub translation_key: String,
	pub parameters: Vec<String>
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct ExitAction {
	pub width: i32,
	pub level: NbtTranslateColor
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct EnchantmentCost {
	pub per_level_above_first: i32,
	pub base: i32
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct Effects {
	#[nbt(rename = "minecraft:attributes")]
	pub attributes: Vec<EnchantmentAttribute>,
	#[nbt(rename = "minecraft:damage")]
	pub damage: Vec<EffectsAttribute>,
	#[nbt(rename = "minecraft:post_attack")]
	pub post_attack: Vec<EffectsAttribute>,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct EnchantmentAttribute {
	pub amount: AttributeModifier,
	pub attribute: String,
	pub id: String,
	pub operation: String,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct AttributeModifierValue {
	pub min_duration: Option<f32>,
	pub max_amplifier: Option<f32>,
	pub min_amplifier: Option<f32>,
	//pub max_duration: Option<AttributeModifier>, // todo
	pub to_apply: Option<String>,
	#[nbt(rename = "type")]
	pub typ: String,
	//pub value: Option<AttributeModifier>,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct AttributeModifier {
	pub base: f32,
	pub per_level_above_first: f32,
	#[nbt(rename = "type")]
	pub typ: String,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct EffectsAttribute {
	pub affected: Option<String>,
	pub effect: AttributeModifierValue,
	pub enchanted: Option<String>,
	pub requirements: EffectRequirements,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct EffectRequirements {
	pub condition: String,
	pub entity: String,
	pub predicate: EffectPredicate
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct EffectPredicate {
	#[nbt(rename = "type")]
	pub typ: String,
	pub tags: Vec<DamageTag>
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct DamageTag {
	pub expected: bool,
	pub id: String,
}

/// Monster spawn light level can either be a single integer value or a range. This handles this disambiguation basically
/// like a union.
#[derive(McDefault, Debug, Clone, PartialEq, McSerialize, McDeserialize)]
pub struct MonsterSpawnLightLevel {
	pub isRange: bool,
	pub level: Option<i32>,
	pub range: Option<MonsterSpawnLightLevelRange>,
}

// custom impl needed because its basically a Union<i32, MonsterSpawnLightLevelRange>
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

impl TryFrom<NbtTag> for MonsterSpawnLightLevel {
	type Error = NbtError;

	fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
		match tag {
			NbtTag::Int(level) => Ok(MonsterSpawnLightLevel {
				isRange: false,
				level: Some(level),
				range: None,
			}),
			NbtTag::Compound(compound) => Ok(MonsterSpawnLightLevel {
				isRange: true,
				level: None,
				range: Some(MonsterSpawnLightLevelRange::try_from(compound)?),
			}),
			_ => Err(NbtError::InvalidType),
		}
	}
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct MonsterSpawnLightLevelRange {
	pub min_inclusive: i32,
	pub max_inclusive: i32,
	#[nbt(rename = "type")]
	pub typ: String,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct WolfVariantAssets {
	pub angry: String,
	pub tame: String,
	pub wild: String
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
				typ: "minecraft:light_range".to_string(),
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