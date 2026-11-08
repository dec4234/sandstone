//! Generate registry packets necessary for a successful login.

use crate::protocol::game::info::registry::{Biome, CatVariant, ChickenVariant, CowVariant, DamageType, DimensionType, PaintingVariant, RegistryDataPacketInternal, RegistryType, ZombieNautilusVariant};
use crate::protocol::game::info::registry::{FrogVariant, PigVariant, RegistryEntry, WolfSoundVariant, WolfVariant};
use crate::protocol::packets::Packet;
use crate::protocol::packets::RegistryDataPacket;
use crate::protocol_types::datatypes::var_types::VarInt;

/// Count the number of tokens provided in macro input.
#[macro_export]
macro_rules! count_items {
    () => { 0 };
    ($_e:expr) => { 1 };
    ($_e:expr, $($rest:expr),*) => {
        1 + $crate::count_items!($($rest),*)
    };
}

/// Create a set of registry packets to be sent to the client during the login sequence.
#[macro_export]
macro_rules! create_registry_packets {
    (
		$($parent_name:literal => {
			$($entry_name:literal, $entry:expr),+
		}),+
	) => {
		vec![
			$(
				Packet::RegistryData(RegistryDataPacket::new(RegistryDataPacketInternal {
					registry_id: $parent_name.to_string(),
					num_entries: VarInt($crate::count_items!($($entry_name),+)),
					entries: vec![$(
						RegistryEntry::new($entry_name.to_string(), Some($entry)),
					)+]
				})),
			)+
		]
	};
}

/// Generate the bare minimum registry packets needed for a successful login.
///
/// Send these packets during the Registry Data phase of the login sequence.
pub fn default() -> Vec<Packet> {
    create_registry_packets!(
		"minecraft:cat_variant" => {
			"minecraft:black", RegistryType::CatVariant(CatVariant::default())
		},
		"minecraft:chicken_variant" => {
			"minecraft:warm", RegistryType::ChickenVariant(ChickenVariant::default())
		},
		"minecraft:cow_variant" => {
			"minecraft:warm", RegistryType::CowVariant(CowVariant::default())
		},
	    "minecraft:damage_type" => {
		    "minecraft:arrow", RegistryType::DamageType(DamageType::new(None, None, 0.1, "arrow".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:bad_respawn_point", RegistryType::DamageType(DamageType::new(Some("intentional_game_design".to_string()), None, 0.1, "badRespawnPoint".to_string(), "always".to_string())),
		    "minecraft:cactus", RegistryType::DamageType(DamageType::new(None, None, 0.1, "cactus".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:campfire", RegistryType::DamageType(DamageType::new(None, Some("burning".to_string()), 0.1, "inFire".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:cramming", RegistryType::DamageType(DamageType::new(None, None, 0.0, "cramming".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:dragon_breath", RegistryType::DamageType(DamageType::new(None, None, 0.0, "dragonBreath".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:drown", RegistryType::DamageType(DamageType::new(None, Some("drowning".to_string()), 0.0, "drown".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:dry_out", RegistryType::DamageType(DamageType::new(None, None, 0.1, "dryout".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:ender_pearl", RegistryType::DamageType(DamageType::new(Some("fall_variants".to_string()), None, 0.0, "fall".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:explosion", RegistryType::DamageType(DamageType::new(None, None, 0.1, "explosion".to_string(), "always".to_string())),
		    "minecraft:fall", RegistryType::DamageType(DamageType::new(Some("fall_variants".to_string()), None, 0.0, "fall".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:falling_anvil", RegistryType::DamageType(DamageType::new(None, None, 0.1, "anvil".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:falling_block", RegistryType::DamageType(DamageType::new(None, None, 0.1, "fallingBlock".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:falling_stalactite", RegistryType::DamageType(DamageType::new(None, None, 0.1, "fallingStalactite".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:fireball", RegistryType::DamageType(DamageType::new(None, Some("burning".to_string()), 0.1, "fireball".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:fireworks", RegistryType::DamageType(DamageType::new(None, None, 0.1, "fireworks".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:fly_into_wall", RegistryType::DamageType(DamageType::new(None, None, 0.0, "flyIntoWall".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:freeze", RegistryType::DamageType(DamageType::new(None, Some("freezing".to_string()), 0.0, "freeze".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:generic", RegistryType::DamageType(DamageType::new(None, None, 0.0, "generic".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:generic_kill", RegistryType::DamageType(DamageType::new(None, None, 0.0, "genericKill".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:hot_floor", RegistryType::DamageType(DamageType::new(None, Some("burning".to_string()), 0.1, "hotFloor".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:in_fire", RegistryType::DamageType(DamageType::new(None, Some("burning".to_string()), 0.1, "inFire".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:in_wall", RegistryType::DamageType(DamageType::new(None, None, 0.0, "inWall".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:indirect_magic", RegistryType::DamageType(DamageType::new(None, None, 0.0, "indirectMagic".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:lava", RegistryType::DamageType(DamageType::new(None, Some("burning".to_string()), 0.1, "lava".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:lightning_bolt", RegistryType::DamageType(DamageType::new(None, None, 0.1, "lightningBolt".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:mace_smash", RegistryType::DamageType(DamageType::new(None, None, 0.1, "mace_smash".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:magic", RegistryType::DamageType(DamageType::new(None, None, 0.0, "magic".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:mob_attack", RegistryType::DamageType(DamageType::new(None, None, 0.1, "mob".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:mob_attack_no_aggro", RegistryType::DamageType(DamageType::new(None, None, 0.1, "mob".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:mob_projectile", RegistryType::DamageType(DamageType::new(None, None, 0.1, "mob".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:on_fire", RegistryType::DamageType(DamageType::new(None, Some("burning".to_string()), 0.0, "onFire".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:out_of_world", RegistryType::DamageType(DamageType::new(None, None, 0.0, "outOfWorld".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:outside_border", RegistryType::DamageType(DamageType::new(None, None, 0.0, "outsideBorder".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:player_attack", RegistryType::DamageType(DamageType::new(None, None, 0.1, "player".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:player_explosion", RegistryType::DamageType(DamageType::new(None, None, 0.1, "explosion.player".to_string(), "always".to_string())),
		    "minecraft:sonic_boom", RegistryType::DamageType(DamageType::new(None, None, 0.0, "sonic_boom".to_string(), "always".to_string())),
		    "minecraft:spear", RegistryType::DamageType(DamageType::new(None, None, 0.1, "spear".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:spit", RegistryType::DamageType(DamageType::new(None, None, 0.1, "mob".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:stalagmite", RegistryType::DamageType(DamageType::new(None, None, 0.0, "stalagmite".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:starve", RegistryType::DamageType(DamageType::new(None, None, 0.0, "starve".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:sting", RegistryType::DamageType(DamageType::new(None, None, 0.1, "sting".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:sweet_berry_bush", RegistryType::DamageType(DamageType::new(None, Some("poking".to_string()), 0.1, "sweetBerryBush".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:thorns", RegistryType::DamageType(DamageType::new(None, Some("thorns".to_string()), 0.1, "thorns".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:thrown", RegistryType::DamageType(DamageType::new(None, None, 0.1, "thrown".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:trident", RegistryType::DamageType(DamageType::new(None, None, 0.1, "trident".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:unattributed_fireball", RegistryType::DamageType(DamageType::new(None, Some("burning".to_string()), 0.1, "onFire".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:wind_charge", RegistryType::DamageType(DamageType::new(None, None, 0.1, "mob".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:wither", RegistryType::DamageType(DamageType::new(None, None, 0.0, "wither".to_string(), "when_caused_by_living_non_player".to_string())),
		    "minecraft:wither_skull", RegistryType::DamageType(DamageType::new(None, None, 0.1, "witherSkull".to_string(), "when_caused_by_living_non_player".to_string()))
	    },
        "minecraft:dimension_type" => {
            "minecraft:overworld", RegistryType::DimensionType(DimensionType::default())
        },
		"minecraft:frog_variant" => {
			"minecraft:warm", RegistryType::FrogVariant(FrogVariant::default())
		},
		"minecraft:painting_variant" => {
			"minecraft:alban", RegistryType::PaintingVariant(PaintingVariant::default())
		},
		"minecraft:pig_variant" => {
			"minecraft:warm", RegistryType::PigVariant(PigVariant::default())
		},
		"minecraft:wolf_sound_variant" => {
			"minecraft:classic", RegistryType::WolfSoundVariant(WolfSoundVariant::default())
		},
		"minecraft:wolf_variant" => {
            "minecraft:woods", RegistryType::WolfVariant(WolfVariant::default())
        },
	    "minecraft:worldgen/biome" => {
		    "minecraft:plains", RegistryType::Biome(Biome::default())
	    },
		"minecraft:zombie_nautilus_variant" => {
			"minecraft:temperate", RegistryType::ZombieNautilusVariant(ZombieNautilusVariant::default())
		}
    )
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
	use std::collections::HashSet;

	fn registries() -> Vec<RegistryDataPacketInternal> {
		default()
			.into_iter()
			.map(|p| match p {
				Packet::RegistryData(r) => r.packet,
				_ => panic!("default() must only return Registry Data packets"),
			})
			.collect()
	}

	#[test]
	fn count_items_counts_each_token() {
		assert_eq!(crate::count_items!(), 0);
		assert_eq!(crate::count_items!("a"), 1);
		assert_eq!(crate::count_items!("a", "b", "c"), 3);
	}

	#[test]
	fn entry_count_matches_entries() {
		// The client reads exactly num_entries entries. A wrong count breaks every later byte.
		for r in registries() {
			assert_eq!(r.num_entries.0 as usize, r.entries.len(), "{}", r.registry_id);
			assert!(!r.entries.is_empty(), "{}", r.registry_id);
		}
	}

	#[test]
	fn ids_are_unique_and_entries_have_data() {
		let regs = registries();
		let mut seen = HashSet::new();
		for r in &regs {
			assert!(seen.insert(r.registry_id.clone()), "duplicate registry {}", r.registry_id);
			let mut entry_ids = HashSet::new();
			for e in &r.entries {
				assert!(entry_ids.insert(e.id.clone()), "duplicate entry {} in {}", e.id, r.registry_id);
				// default() builds every entry with Some(data). An entry with no data tells the
				// client to use its own copy from a known pack, which this server does not rely on.
				assert!(e.is_present && e.data.is_some(), "entry {} in {}", e.id, r.registry_id);
			}
		}
	}

	#[test]
	fn includes_registries_needed_for_login() {
		// The client disconnects during configuration if one of these is missing.
		let ids: HashSet<String> = registries().into_iter().map(|r| r.registry_id).collect();
		for needed in [
			"minecraft:dimension_type",
			"minecraft:worldgen/biome",
			"minecraft:damage_type",
			"minecraft:painting_variant",
			"minecraft:wolf_variant",
			"minecraft:wolf_sound_variant",
			"minecraft:cat_variant",
			"minecraft:chicken_variant",
			"minecraft:cow_variant",
			"minecraft:frog_variant",
			"minecraft:pig_variant",
			"minecraft:zombie_nautilus_variant",
		] {
			assert!(ids.contains(needed), "missing {needed}");
		}
	}

	#[test]
	fn every_registry_round_trips() {
		// Decoding uses the registry id to pick the entry type, so this also checks that the id
		// matches the type of data we put in it.
		for r in registries() {
			let mut s = McSerializer::new();
			r.mc_serialize(&mut s).unwrap();
			let mut d = McDeserializer::new(s.as_bytes());
			let back = RegistryDataPacketInternal::mc_deserialize(&mut d).unwrap_or_else(|e| panic!("{}: {e:?}", r.registry_id));
			assert_eq!(back, r);
			assert!(d.is_at_end(), "{}", r.registry_id);
		}
	}
}
