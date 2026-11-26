//! Complex and nested registry components. Like biome effects, music data, etc.

use crate::protocol::game::info::registry::McDeserialize;
use crate::protocol::game::info::registry::McDeserializer;
use crate::protocol::game::info::registry::McSerialize;
use crate::protocol::game::info::registry::McSerializer;
use crate::protocol::game::info::registry::SerializingErr;
use crate::protocol::game::info::registry::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
use crate::protocol_types::datatypes::nbt::{NbtCompound, NbtTag};
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{AsNbt, FromNbt, McDefault, McDeserialize};
use serde::{Deserialize, Serialize};

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct PigSounds {
	pub ambient_sound: String,
	pub death_sound: String,
	pub hurt_sound: String,
	pub step_sound: String,
	pub eat_sound: String,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct CatSounds {
	pub hurt_sound: String,
	pub purr_sound: String,
	pub eat_sound: String,
	pub hiss_sound: String,
	pub ambient_sound: String,
	pub beg_for_food_sound: String,
	pub death_sound: String,
	pub purreow_sound: String,
	pub stray_ambient_sound: String,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct WolfSounds {
	pub ambient_sound: String,
	pub death_sound: String,
	pub growl_sound: String,
	pub hurt_sound: String,
	pub pant_sound: String,
	pub step_sound: String,
	pub whine_sound: String,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct ChickenSounds {
	pub ambient_sound: String,
	pub death_sound: String,
	pub hurt_sound: String,
	pub step_sound: String,
}

/// Used for some sections of registry components such as painting_variant
#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct NbtTranslateColor {
	pub color: Option<String>,
	pub translate: String,
}

impl From<NbtTag> for Option<NbtTranslateColor> {
	fn from(tag: NbtTag) -> Self {
		NbtTranslateColor::try_from(tag).ok()
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
	pub weight: i32,
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
	pub parameters: Vec<String>,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct ExitAction {
	pub label: NbtTranslateColor,
	pub width: i32,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct EnchantmentCost {
	pub per_level_above_first: i32,
	pub base: i32,
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
	pub predicate: EffectPredicate,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct EffectPredicate {
	#[nbt(rename = "type")]
	pub typ: String,
	pub tags: Vec<DamageTag>,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct DamageTag {
	pub expected: bool,
	pub id: String,
}

/// Monster spawn light level is either a single integer value or a range.
#[derive(McDefault, Debug, Clone, PartialEq, McSerialize, McDeserialize)]
#[repr(i32)]
pub enum MonsterSpawnLightLevel {
	Level(i32) = 0,
	Range(MonsterSpawnLightLevelRange) = 1,
}

impl From<MonsterSpawnLightLevel> for NbtTag {
	fn from(value: MonsterSpawnLightLevel) -> Self {
		match value {
			MonsterSpawnLightLevel::Level(level) => NbtTag::Int(level),
			MonsterSpawnLightLevel::Range(range) => NbtTag::Compound(range.into()),
		}
	}
}

impl TryFrom<NbtTag> for MonsterSpawnLightLevel {
	type Error = NbtError;

	fn try_from(tag: NbtTag) -> Result<Self, Self::Error> {
		match tag {
			NbtTag::Int(level) => Ok(MonsterSpawnLightLevel::Level(level)),
			NbtTag::Compound(compound) => Ok(MonsterSpawnLightLevel::Range(MonsterSpawnLightLevelRange::try_from(compound)?)),
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
	pub wild: String,
}

/// Used for "minecraft:sulfur_cube_archetype" registry component
#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct SulfurCubeAttributeModifier {
	pub amount: f64,
	pub attribute: String,
	pub id: String,
	pub operation: String,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct SulfurCubeContactDamage {
	pub amount: f32,
	pub attribute_to_source: i8,
	pub damage_type: String,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct SulfurCubeExplosion {
	pub causes_fire: i8,
	pub fuse: i32,
	pub power: i32,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct SulfurCubeKnockbackModifiers {
	pub horizontal_power: f32,
	pub vertical_power: f32,
}

#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct SulfurCubeSoundSettings {
	pub hit_sound: String,
	pub push_sound: String,
	pub push_sound_cooldown: f32,
	pub push_sound_impulse_threshold: f32,
}

/// Used for "minecraft:block_transformer" registry component. One rule, e.g. an axe that strips a log.
#[derive(McDefault, Debug, Clone, PartialEq, Deserialize, Serialize, AsNbt, FromNbt, McSerialize, McDeserialize)]
pub struct BlockTransform {
	/// Same format as a "minecraft:worldgen/block_state_provider" entry. The shape changes with its
	/// `type` field, so it is kept as raw NBT.
	pub block_state_provider: NbtCompound,
	pub sound: String,
	pub item_damage_per_use: i32,
	pub particle: Option<String>,
	pub update_from_neighbors: Option<i8>,
	pub transform_type: Option<String>,
	pub disallowed_faces: Option<Vec<String>>,
	pub drop_strategy: Option<String>,
	pub loot: Option<String>,
}

/// Registry body for "minecraft:block_transformer".
///
/// The root NBT tag of an entry is a list of rules, not a compound, so `registry_entry!` cannot make
/// this type.
#[derive(McDefault, Debug, Clone, PartialEq)]
pub struct BlockTransformer {
	pub transforms: Vec<BlockTransform>,
}

impl BlockTransformer {
	pub fn new(transforms: Vec<BlockTransform>) -> Self {
		Self {
			transforms,
		}
	}
}

impl McSerialize for BlockTransformer {
	/// Serialize as a network NBT list.
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		NbtTag::from(self.transforms.clone()).mc_serialize(serializer)
	}
}

impl McDeserialize for BlockTransformer {
	/// Deserialize from a network NBT list.
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let tag = NbtTag::mc_deserialize(deserializer)?;
		let transforms = Vec::<BlockTransform>::try_from(tag).map_err(|e| SerializingErr::DeserializationError(format!("in registry struct 'BlockTransformer': {}", e)))?;
		Ok(Self {
			transforms,
		})
	}
}

/// Registry body for "minecraft:worldgen/block_state_provider".
///
/// The fields change with the `type` field (rule_based, noise_threshold_provider, etc.), so the whole
/// compound is kept as raw NBT.
#[derive(McDefault, Debug, Clone, PartialEq)]
pub struct BlockStateProvider {
	pub provider: NbtCompound,
}

impl BlockStateProvider {
	pub fn new(provider: NbtCompound) -> Self {
		Self {
			provider,
		}
	}
}

impl McSerialize for BlockStateProvider {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.provider.mc_serialize(serializer)
	}
}

impl McDeserialize for BlockStateProvider {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		Ok(Self {
			provider: NbtCompound::mc_deserialize(deserializer)?,
		})
	}
}

#[cfg(test)]
mod tests {
    use crate::nbt_compound;
    use crate::protocol::game::info::registry::registry_components::*;
    use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
    use crate::protocol::testing::McDefault;
    use crate::protocol_types::datatypes::nbt::nbt_error::NbtError;
    use crate::protocol_types::datatypes::nbt::{NbtCompound, NbtTag};

    /// Shape of the vanilla 26.3 "minecraft:hoe" entry: optional fields present, others absent.
	fn hoe_transform() -> BlockTransform {
		let mut provider = NbtCompound::new_no_name();
		provider.add("type", "minecraft:rule_based");
		provider.add("rules", Vec::<NbtCompound>::new());
		BlockTransform {
			block_state_provider: provider,
			sound: "minecraft:item.hoe.till".to_string(),
			item_damage_per_use: 1,
			particle: None,
			update_from_neighbors: Some(1),
			transform_type: None,
			disallowed_faces: Some(vec!["down".to_string()]),
			drop_strategy: None,
			loot: Some("minecraft:blocks/rooted_dirt".to_string()),
		}
	}

	/// Vanilla sends a block_transformer entry with a List root tag (id 9), not a Compound. The
	/// collector failed on exactly this, so the wire form must stay a list and read back unchanged.
	#[test]
	fn block_transformer_is_list_root_and_round_trips() {
		let transformer = BlockTransformer::new(vec![hoe_transform(), hoe_transform()]);
		let mut serializer = McSerializer::new();
		transformer.mc_serialize(&mut serializer).unwrap();
		assert_eq!(serializer.as_bytes()[0], 9, "root tag must be a list");

		let mut deserializer = McDeserializer::new(serializer.as_bytes());
		let back = BlockTransformer::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(back, transformer);
		assert!(deserializer.is_at_end());
	}

	/// A compound root is wrong for this registry and must be an error, not a silent empty value.
	#[test]
	fn block_transformer_rejects_compound_root() {
		let mut serializer = McSerializer::new();
		hoe_transform().as_nbt().mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(serializer.as_bytes());
		assert!(BlockTransformer::mc_deserialize(&mut deserializer).is_err());
	}

	/// Each provider `type` has different fields. All of them must survive, because the client uses
	/// them for world generation.
	#[test]
	fn block_state_provider_keeps_type_specific_fields() {
		let mut provider = NbtCompound::new_no_name();
		provider.add("type", "minecraft:noise_threshold_provider");
		provider.add("seed", NbtTag::Long(2345));
		provider.add("scale", NbtTag::Float(0.005));
		provider.add("threshold", NbtTag::Float(-0.8));
		provider.add("high_chance", NbtTag::Float(0.33333334));
		provider.add("default_state", "minecraft:dandelion");
		let provider = BlockStateProvider::new(provider);

		let mut serializer = McSerializer::new();
		provider.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(serializer.as_bytes());
		assert_eq!(BlockStateProvider::mc_deserialize(&mut deserializer).unwrap(), provider);
		assert!(deserializer.is_at_end());
	}

	/// Test that MonsterSpawnLightLevel can either be a struct or an integer
	#[test]
	fn test_monster_light_level_format() {
		let mut compound = NbtCompound::new_no_name();
		let light = MonsterSpawnLightLevel::Range(MonsterSpawnLightLevelRange {
			min_inclusive: 0,
			max_inclusive: 15,
			typ: "minecraft:light_range".to_string(),
		});
		compound.add("light".to_string(), light);
		let light_num = MonsterSpawnLightLevel::Level(10);
		compound.add("light_num".to_string(), light_num);

		println!("{}", serde_json::to_string(&compound).unwrap());

		let mut serializer = McSerializer::new();
		compound.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(serializer.as_bytes());
		let deserialized_compound = NbtCompound::mc_deserialize(&mut deserializer).unwrap();

		assert_eq!(compound, deserialized_compound);
	}

	/// Registry components travel as NBT. Each one built from `mc_default` must survive
	/// `as_nbt` -> NBT wire bytes -> `TryFrom<NbtCompound>` with no change.
	macro_rules! assert_nbt_round_trip {
		($($t:ty),* $(,)?) => {
			$(
				let original = <$t as McDefault>::mc_default();
				let mut serializer = McSerializer::new();
				original.as_nbt().mc_serialize(&mut serializer).unwrap();
				let mut deserializer = McDeserializer::new(serializer.as_bytes());
				let nbt = NbtCompound::mc_deserialize(&mut deserializer).unwrap();
				let back = <$t>::try_from(nbt).unwrap_or_else(|e| panic!("{}: {:?}", stringify!($t), e));
				assert_eq!(original, back, "{} changed after an NBT round trip", stringify!($t));
			)*
		};
	}

	#[test]
	fn components_nbt_round_trip() {
		assert_nbt_round_trip!(
			PigSounds,
			CatSounds,
			ChickenSounds,
			NbtTranslateColor,
			BiomeEffects,
			BiomeMood,
			BiomeMusicData,
			BiomeMusic,
			ChatTypePart,
			ExitAction,
			EnchantmentCost,
			Effects,
			EnchantmentAttribute,
			AttributeModifierValue,
			AttributeModifier,
			EffectsAttribute,
			EffectRequirements,
			EffectPredicate,
			DamageTag,
			MonsterSpawnLightLevelRange,
			WolfVariantAssets,
			WolfSounds,
			SulfurCubeAttributeModifier,
			SulfurCubeContactDamage,
			SulfurCubeExplosion,
			SulfurCubeKnockbackModifiers,
			SulfurCubeSoundSettings,
		);
	}

	/// Vanilla uses namespaced and reserved-word keys. The `#[nbt(rename)]` keys must be on the wire,
	/// not the Rust field names, or the client cannot read the registry.
	#[test]
	fn renamed_keys_are_used() {
		let effects = Effects::mc_default().as_nbt();
		assert!(effects.get("minecraft:attributes").is_some());
		assert!(effects.get("minecraft:damage").is_some());
		assert!(effects.get("minecraft:post_attack").is_some());
		assert!(effects.get("attributes").is_none());

		let range = MonsterSpawnLightLevelRange::mc_default().as_nbt();
		assert!(range.get("type").is_some());
		assert!(range.get("typ").is_none());
	}

	/// A required field that is absent must be an error, so bad registry data is not silently accepted.
	#[test]
	fn missing_required_field_errors() {
		let mut nbt = WolfVariantAssets::mc_default().as_nbt();
		nbt.remove("tame");
		assert!(matches!(WolfVariantAssets::try_from(nbt), Err(NbtError::MissingField(_))));
	}

	/// An absent optional field is `None`, not an error. Vanilla leaves out `color` on many entries.
	#[test]
	fn missing_optional_field_is_none() {
		let nbt = nbt_compound! { "translate" => "painting.minecraft.alban.title" };
		let color = NbtTranslateColor::try_from(nbt).unwrap();
		assert_eq!(color.color, None);
		assert_eq!(color.translate, "painting.minecraft.alban.title");
	}

	#[test]
	fn optional_translate_color_from_tag() {
		let good: NbtTag = NbtTranslateColor::mc_default().into();
		assert_eq!(Option::<NbtTranslateColor>::from(good), Some(NbtTranslateColor::mc_default()));
		assert_eq!(Option::<NbtTranslateColor>::from(NbtTag::Int(1)), None);
	}

	/// The vanilla value is either a plain int light level or a range compound.
	#[test]
	fn monster_light_level_tag_conversions() {
		let single = MonsterSpawnLightLevel::try_from(NbtTag::Int(7)).unwrap();
		assert_eq!(single, MonsterSpawnLightLevel::Level(7));
		assert_eq!(NbtTag::from(single), NbtTag::Int(7));

		let range = MonsterSpawnLightLevelRange::mc_default();
		let ranged = MonsterSpawnLightLevel::try_from(NbtTag::Compound(range.as_nbt())).unwrap();
		assert_eq!(ranged, MonsterSpawnLightLevel::Range(range.clone()));
		assert_eq!(NbtTag::from(ranged), NbtTag::Compound(range.as_nbt()));

		assert!(matches!(MonsterSpawnLightLevel::try_from(NbtTag::String("7".into())), Err(NbtError::InvalidType)));
	}
}
