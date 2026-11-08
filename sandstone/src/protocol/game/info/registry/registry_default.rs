//! Define the default values for the different registry entries according to the defaults generally
//! provided by Minecraft. See the 1.21 defaults here https://gist.github.com/Mansitoh/e6c5cf8bbf17e9faf4e4e75bb3f4789d

use crate::nbt_compound;
use crate::protocol::game::info::registry::registry_components::{CatSounds, ChickenSounds, MonsterSpawnLightLevel, NbtTranslateColor, PigSounds, WolfVariantAssets};
use crate::protocol::game::info::registry::{
	BannerPattern, Biome, CatSoundVariant, CatVariant, ChickenSoundVariant, ChickenVariant, CowVariant, DimensionType, FrogVariant, PaintingVariant, PigSoundVariant, PigVariant, WolfSoundVariant,
	WolfVariant, ZombieNautilusVariant,
};
use crate::protocol_types::datatypes::nbt::NbtCompound;

impl Default for BannerPattern {
	fn default() -> Self {
		Self {
			asset_id: "minecraft:base".to_string(),
			translation_key: "block.minecraft.banner.base".to_string(),
		}
	}
}

impl Default for Biome {
	/// Matches "minecraft:plains".
	fn default() -> Self {
		Self {
			attributes: Some(nbt_compound! {
				"minecraft:visual/sky_color" => "#78a7ff"
			}),
			downfall: 0.4,
			effects: nbt_compound! {
				"water_color" => "#3f76e4"
			},
			has_precipitation: true,
			temperature: 0.8,
			temperature_modifier: None,
		}
	}
}

impl Default for CatSoundVariant {
	fn default() -> Self {
		Self {
			baby_sounds: CatSounds {
				hurt_sound: "minecraft:entity.baby_cat.hurt".to_string(),
				purr_sound: "minecraft:entity.baby_cat.purr".to_string(),
				eat_sound: "minecraft:entity.baby_cat.eat".to_string(),
				hiss_sound: "minecraft:entity.baby_cat.hiss".to_string(),
				ambient_sound: "minecraft:entity.baby_cat.ambient".to_string(),
				beg_for_food_sound: "minecraft:entity.baby_cat.beg_for_food".to_string(),
				death_sound: "minecraft:entity.baby_cat.death".to_string(),
				purreow_sound: "minecraft:entity.baby_cat.purreow".to_string(),
				stray_ambient_sound: "minecraft:entity.baby_cat.stray_ambient".to_string(),
			},
			adult_sounds: CatSounds {
				hurt_sound: "minecraft:entity.cat.hurt".to_string(),
				purr_sound: "minecraft:entity.cat.purr".to_string(),
				eat_sound: "minecraft:entity.cat.eat".to_string(),
				hiss_sound: "minecraft:entity.cat.hiss".to_string(),
				ambient_sound: "minecraft:entity.cat.ambient".to_string(),
				beg_for_food_sound: "minecraft:entity.cat.beg_for_food".to_string(),
				death_sound: "minecraft:entity.cat.death".to_string(),
				purreow_sound: "minecraft:entity.cat.purreow".to_string(),
				stray_ambient_sound: "minecraft:entity.cat.stray_ambient".to_string(),
			},
		}
	}
}

impl Default for CatVariant {
	fn default() -> Self {
		Self {
			asset_id: "minecraft:entity/cat/black".to_string(),
		}
	}
}

impl Default for ChickenSoundVariant {
	fn default() -> Self {
		Self {
			baby_sounds: ChickenSounds {
				hurt_sound: "minecraft:entity.baby_chicken.hurt".to_string(),
				ambient_sound: "minecraft:entity.baby_chicken.ambient".to_string(),
				death_sound: "minecraft:entity.baby_chicken.death".to_string(),
				step_sound: "minecraft:entity.baby_chicken.step".to_string(),
			},
			adult_sounds: ChickenSounds {
				hurt_sound: "minecraft:entity.chicken.hurt".to_string(),
				ambient_sound: "minecraft:entity.chicken.ambient".to_string(),
				death_sound: "minecraft:entity.chicken.death".to_string(),
				step_sound: "minecraft:entity.chicken.step".to_string(),
			},
		}
	}
}

impl Default for ChickenVariant {
	fn default() -> Self {
		Self {
			asset_id: "minecraft:entity/chicken/warm_chicken".to_string(),
			model: None,
		}
	}
}

impl Default for CowVariant {
	fn default() -> Self {
		Self {
			asset_id: "minecraft:entity/cow/warm_cow".to_string(),
			model: Some("warm".to_string()),
		}
	}
}

impl Default for DimensionType {
	fn default() -> Self {
		Self {
			ambient_light: 0.0,
			attributes: NbtCompound::new_no_name(),
			coordinate_scale: 1.0,
			has_ceiling: 0,
			has_fixed_time: None,
			has_skylight: 1,
			height: 384,
			infiniburn: "#minecraft:infiniburn_overworld".to_string(),
			logical_height: 384,
			min_y: -64,
			monster_spawn_block_light_limit: 0,
			monster_spawn_light_level: MonsterSpawnLightLevel::default(),
			skybox: None,
			timelines: "#minecraft:in_overworld".to_string(),
		}
	}
}

impl Default for FrogVariant {
	fn default() -> Self {
		Self {
			asset_id: "minecraft:entity/frog/warm_frog".to_string(),
		}
	}
}

impl Default for MonsterSpawnLightLevel {
	fn default() -> Self {
		Self {
			isRange: false,
			level: Some(7),
			range: None,
		}
	}
}

impl Default for PaintingVariant {
	fn default() -> Self {
		Self {
			asset_id: "minecraft:alban".to_string(),
			author: Some(NbtTranslateColor {
				translate: "painting.minecraft.alban.author".to_string(),
				color: Some("gray".to_string()),
			}),
			height: 1,
			title: NbtTranslateColor {
				translate: "painting.minecraft.alban.title".to_string(),
				color: Some("yellow".to_string()),
			},
			width: 1,
		}
	}
}

impl Default for PigSoundVariant {
	fn default() -> Self {
		Self {
			baby_sounds: PigSounds {
				death_sound: "minecraft:entity.baby_pig.death".to_string(),
				hurt_sound: "minecraft:entity.baby_pig.hurt".to_string(),
				ambient_sound: "minecraft:entity.baby_pig.ambient".to_string(),
				eat_sound: "minecraft:entity.baby_pig.eat".to_string(),
				step_sound: "minecraft:entity.baby_pig.step".to_string(),
			},
			adult_sounds: PigSounds {
				death_sound: "minecraft:entity.pig_big.death".to_string(),
				hurt_sound: "minecraft:entity.pig_big.hurt".to_string(),
				ambient_sound: "minecraft:entity.pig_big.ambient".to_string(),
				eat_sound: "minecraft:entity.pig_big.eat".to_string(),
				step_sound: "minecraft:entity.pig.step".to_string(),
			},
		}
	}
}

impl Default for PigVariant {
	fn default() -> Self {
		Self {
			model: None,
			asset_id: "minecraft:entity/pig/warm_pig".to_string(),
		}
	}
}

impl Default for WolfSoundVariant {
	fn default() -> Self {
		Self {
			ambient_sound: "minecraft:entity.wolf.ambient".to_string(),
			death_sound: "minecraft:entity.wolf.death".to_string(),
			growl_sound: "minecraft:entity.wolf.growl".to_string(),
			hurt_sound: "minecraft:entity.wolf.hurt".to_string(),
			pant_sound: "minecraft:entity.wolf.pant".to_string(),
			whine_sound: "minecraft:entity.wolf.whine".to_string(),
		}
	}
}

impl Default for WolfVariant {
	fn default() -> Self {
		Self {
			assets: WolfVariantAssets {
				wild: "minecraft:entity/wolf/wolf_woods".to_string(),
				tame: "minecraft:entity/wolf/wolf_woods_tame".to_string(),
				angry: "minecraft:entity/wolf/wolf_woods_angry".to_string(),
			},
		}
	}
}

impl Default for ZombieNautilusVariant {
	fn default() -> Self {
		Self {
			model: None,
			asset_id: "minecraft:entity/nautilus/zombie_nautilus".to_string(),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::protocol::game::info::registry::registry_components::MonsterSpawnLightLevel;
	use crate::protocol::game::info::registry::{
		BannerPattern, Biome, CatSoundVariant, CatVariant, ChickenSoundVariant, ChickenVariant, CowVariant, DimensionType, FrogVariant, PaintingVariant, PigSoundVariant, PigVariant, WolfSoundVariant,
		WolfVariant, ZombieNautilusVariant,
	};
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};

	/// A server can send these defaults as registry data. Each one must survive the real wire path
	/// (NBT serialize, then deserialize) with no change.
	macro_rules! assert_default_round_trip {
		($($t:ty),* $(,)?) => {
			$(
				let original = <$t>::default();
				let mut serializer = McSerializer::new();
				original.mc_serialize(&mut serializer).unwrap();
				let mut deserializer = McDeserializer::new(serializer.as_bytes());
				let back = <$t>::mc_deserialize(&mut deserializer).unwrap_or_else(|e| panic!("{}: {:?}", stringify!($t), e));
				assert_eq!(original, back, "{} default changed after a round trip", stringify!($t));
				assert!(deserializer.is_at_end(), "{} left unread bytes", stringify!($t));
			)*
		};
	}

	#[test]
	fn defaults_round_trip() {
		assert_default_round_trip!(
			BannerPattern,
			Biome,
			CatSoundVariant,
			CatVariant,
			ChickenSoundVariant,
			ChickenVariant,
			CowVariant,
			DimensionType,
			FrogVariant,
			PaintingVariant,
			PigSoundVariant,
			PigVariant,
			WolfSoundVariant,
			WolfVariant,
			ZombieNautilusVariant,
		);
	}

	/// The client rejects a dimension type that breaks these rules
	/// (https://minecraft.wiki/w/Dimension_type): height and min_y are multiples of 16,
	/// the world fits in -2032..=2031, and logical_height is not more than height.
	#[test]
	fn default_dimension_type_is_valid() {
		let dim = DimensionType::default();
		assert_eq!(dim.height % 16, 0);
		assert_eq!(dim.min_y % 16, 0);
		assert!(dim.min_y >= -2032);
		assert!(dim.min_y + dim.height <= 2032);
		assert!(dim.logical_height <= dim.height);
		assert!(dim.infiniburn.starts_with('#'), "infiniburn is a block tag");
	}

	/// The default monster spawn light level is a single value, so it must encode as an int.
	#[test]
	fn default_monster_spawn_light_level_is_single_value() {
		let level = MonsterSpawnLightLevel::default();
		assert!(!level.isRange);
		assert!(level.range.is_none());
		assert!(matches!(level.level, Some(0..=15)));
	}

	/// Baby and adult sounds must not be swapped or mixed.
	#[test]
	fn baby_and_adult_sounds_are_separate() {
		let cat = CatSoundVariant::default();
		for s in [&cat.baby_sounds.hurt_sound, &cat.baby_sounds.ambient_sound, &cat.baby_sounds.death_sound] {
			assert!(s.starts_with("minecraft:entity.baby_cat."), "{s}");
		}
		for s in [&cat.adult_sounds.hurt_sound, &cat.adult_sounds.ambient_sound, &cat.adult_sounds.death_sound] {
			assert!(s.starts_with("minecraft:entity.cat."), "{s}");
		}

		let chicken = ChickenSoundVariant::default();
		assert!(chicken.baby_sounds.step_sound.starts_with("minecraft:entity.baby_chicken."));
		assert!(chicken.adult_sounds.step_sound.starts_with("minecraft:entity.chicken."));

		let pig = PigSoundVariant::default();
		assert!(pig.baby_sounds.hurt_sound.starts_with("minecraft:entity.baby_pig."));
		assert!(!pig.adult_sounds.hurt_sound.contains("baby"));
	}
}
