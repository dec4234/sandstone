//! Define the default values for the different registry entries according to the defaults generally
//! provided by Minecraft. See the 1.21 defaults here https://gist.github.com/Mansitoh/e6c5cf8bbf17e9faf4e4e75bb3f4789d

use crate::protocol::game::info::registry::{BannerPattern, CatVariant, ChickenVariant, CowVariant, DimensionType, FrogVariant, PigVariant, WolfSoundVariant, WolfVariant};

impl Default for DimensionType {
	fn default() -> Self {
		Self {
			fixed_time: Some(1000),
			ambient_light: 0.0,
			bed_works: 1,
			coordinate_scale: 1.0,
			effects: "minecraft:overworld".to_string(),
			has_ceiling: 0,
			has_raids: 1,
			has_skylight: 1,
			height: 384,
			infiniburn: "#minecraft:infiniburn_overworld".to_string(),
			logical_height: 384,
			min_y: -64,
			monster_spawn_block_light_limit: 0,
			monster_spawn_light_level: 0,
			natural: 1,
			piglin_safe: 0,
			respawn_anchor_works: 0,
			ultrawarm: 0,
		}
	}
}

impl Default for BannerPattern {
	fn default() -> Self {
		Self {
			asset_id: "minecraft:base".to_string(),
			translation_key: "block.minecraft.banner.base".to_string(),
		}
	}
}

impl Default for WolfVariant {
	fn default() -> Self {
		Self {
			wild_texture: "minecraft:entity/wolf/wolf_woods".to_string(),
			tame_texture: "minecraft:entity/wolf/wolf_woods_tame".to_string(),
			angry_texture: "minecraft:entity/wolf/wolf_woods_angry".to_string(),
			biomes: "minecraft:forest".to_string(),
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
			pant_sound: "minecraft:entity.wolf.pant".to_string(),
			hurt_sound: "minecraft:entity.wolf.hurt".to_string(),
			growl_sound: "minecraft:entity.wolf.growl".to_string(),
			whine_sound: "minecraft:entity.wolf.whine".to_string(),
			death_sound: "minecraft:entity.wolf.death".to_string(),
			ambient_sound: "minecraft:entity.wolf.ambient".to_string(),
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

impl Default for CatVariant {
	fn default() -> Self {
		Self {
			asset_id: "minecraft:entity/cat/black".to_string(),
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

impl Default for ChickenVariant {
	fn default() -> Self {
		Self {
			asset_id: "minecraft:entity/chicken/warm_chicken".to_string(),
			model: None,
		}
	}
}
