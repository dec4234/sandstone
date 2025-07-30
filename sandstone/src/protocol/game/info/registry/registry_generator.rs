//! Generate registry packets necessary for a successful login.

use crate::protocol::game::info::registry::{CatVariant, ChickenVariant, CowVariant, DimensionType, PaintingVariant, RegistryDataPacketInternal, RegistryType};
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
        }
    )
}
