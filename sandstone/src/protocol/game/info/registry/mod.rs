#![allow(non_snake_case)]

//! Registry data structures for specific details about biomes, dimensions, datapacks, etc.
//! https://minecraft.wiki/w/Java_Edition_protocol/Registry_data

use crate::protocol::game::info::registry::registry_components::NbtTranslateColor;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::registry_entry;
use sandstone_derive::{McDefault, McSerialize};

pub mod registry_default;
pub mod registry_generator;
pub mod registry_components;

#[derive(McDefault, Debug, Clone, PartialEq)]
pub struct RegistryDataPacketInternal {
	/// The registry type this data is for, e.g. "minecraft:dimension_type"
	pub registry_id: String,
	pub num_entries: VarInt, // can't use PrefixedArray because we have custom deserialization
	pub entries: Vec<RegistryEntry>
}

impl McSerialize for RegistryDataPacketInternal {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.registry_id.mc_serialize(serializer)?;
		self.num_entries.mc_serialize(serializer)?;
		self.entries.mc_serialize(serializer)?;
		Ok(())
	}
}

impl McDeserialize for RegistryDataPacketInternal {
	/// Deserialize the registry data packet according to the number of entries specified.
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let id = String::mc_deserialize(deserializer)?;
		let num_entries = VarInt::mc_deserialize(deserializer)?;

		let mut entries = Vec::with_capacity(num_entries.0 as usize);

		for _ in 0..num_entries.0 {
			// We need to deserialize each entry, but we don't know the type yet.
			// So we will deserialize it as a RegistryEntry and then convert it to the correct type.
			let entry = RegistryEntry::mc_deserialize(deserializer, id.clone())?;
			entries.push(entry);
		}

		Ok(Self {
			registry_id: id,
			num_entries,
			entries,
		})
	}
}

#[derive(McDefault, McSerialize, Debug, Clone, PartialEq)]
pub struct RegistryEntry {
	/// The ID of the registry entry, e.g. "minecraft:overworld"
	pub id: String,
	/// Whether the entry is present in the registry, used for serializing the data field.
	pub is_present: bool,
	pub data: Option<RegistryType>,
}

impl RegistryEntry {
	pub fn new(id: String, data: Option<RegistryType>) -> Self {
		Self {
			id,
			is_present: data.is_some(),
			data,
		}
	}

	pub fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer, registry_type: String) -> SerializingResult<'a, Self> {
		let id = String::mc_deserialize(deserializer)?;
		let is_present = bool::mc_deserialize(deserializer)?;

		let data = if is_present {
			let data = RegistryType::deserialize(deserializer, registry_type)?;
			Some(data)
		} else {
			None
		};

		Ok(Self {
			id,
			is_present,
			data,
		})
	}
}

/// Define a registry data group sent by a RegistryDataPacket.
#[macro_export]
macro_rules! registry_entry {
	(
		$(
		$mc_name:literal, $lib_name:ident => {
			$( $field_name:ident : $field_type:ty ),*
		}
		),*
	) => {
		$(
			#[derive(McDefault, Debug, Clone, PartialEq)]
			pub struct $lib_name {
				$(
					pub $field_name: $field_type,
				)*
			}

			impl $lib_name {
				pub fn new($($field_name: $field_type),*) -> Self {
					Self {
						$($field_name,)*
					}
				}

				/// Convert from NBT to the registry entry. Useful for deserializing from a RegistryDataPacket.
				pub fn from_nbt(nbt: &NbtCompound) -> Result<Self, SerializingErr> {
					Ok(Self {
						$(
							$field_name: nbt.map.get(stringify!($field_name))
								.ok_or_else(|| SerializingErr::NbtMissingField(stringify!($field_name).to_string()))?
								.clone().into(),
						)*
					})
				}

				/// Convert the registry entry to NBT. Useful for serializing to a RegistryDataPacket.
				pub fn to_nbt(&self) -> NbtCompound {
					let mut nbt = NbtCompound::new::<String>(None);

					$(
						nbt.add(stringify!($field_name), self.$field_name.clone());
					)*

					nbt
				}
			}

			impl McSerialize for $lib_name {
				/// Serialize the registry via NBT.
				fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
					self.to_nbt().mc_serialize(serializer)
				}
			}

			impl McDeserialize for $lib_name {
				/// Deserialize the registry via NBT.
				fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
					let nbt = NbtCompound::mc_deserialize(deserializer)?;
					Self::from_nbt(&nbt)
				}
			}
		)*

		#[derive(McDefault, Debug, Clone, PartialEq)]
		pub enum RegistryType {
			$($lib_name($lib_name)),*
		}

		impl RegistryType {
			/// Deserialize the registry type according to the given registry type string. Such as "minecraft:dimension_type".
			pub fn deserialize<'a>(deserializer: &'a mut McDeserializer, registry_type: String) -> SerializingResult<'a, Self> {
				match registry_type.as_str() {
					$(
						$mc_name => {
							let entry = $lib_name::mc_deserialize(deserializer)?;
							Ok(RegistryType::$lib_name(entry))
						}
					),*
					_ => Err(SerializingErr::UniqueFailure(format!("Unknown registry type: {}", registry_type))),
				}
			}
		}

		impl McSerialize for RegistryType {
			fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
				match self {
					$(RegistryType::$lib_name(entry) => entry.mc_serialize(serializer)),*
				}
			}
		}
	};
}

/// Used to convert a field defined for a registry structure into a `RegistryEntry`. This requires a
/// PrefixedOptional, and we don't want to double wrap any Option<T> fields like PrefixedOptional<Option<T>>.
#[macro_export]
macro_rules! registry_entry_optional {
	(Option<$t:ty>, $field:expr) => {
		$field.clone().map(Into::into)
	};
	($other:ty, $field:expr) => {
		Some($field.clone().into())
	};
}

//todo: https://minecraft.wiki/w/Mob_variant_definitions#Cat

// https://minecraft.wiki/w/Java_Edition_protocol/Registry_data
registry_entry!(
	"minecraft:banner_pattern", BannerPattern => {
		asset_id: String,
		translation_key: String
	},
	"minecraft:cat_variant", CatVariant => {
		asset_id: String
	},
	"minecraft:chicken_variant", ChickenVariant => {
		asset_id: String,
		model: Option<String>
	},
	"minecraft:cow_variant", CowVariant => {
		asset_id: String,
		model: Option<String>
	},
	"minecraft:dimension_type", DimensionType => {
		fixed_time: Option<i64>,
		has_skylight: i8,
		has_ceiling: i8,
		ultrawarm: i8,
		natural: i8,
		coordinate_scale: f64,
		bed_works: i8,
		respawn_anchor_works: i8,
		min_y: i32,
		height: i32,
		logical_height: i32,
		infiniburn: String,
		effects: String,
		ambient_light: f32,
		piglin_safe: i8,
		has_raids: i8,
		monster_spawn_light_level: i32,
		monster_spawn_block_light_limit: i32
	},
	"minecraft:frog_variant", FrogVariant => {
		asset_id: String
	},
	"minecraft:painting_variant", PaintingVariant => {
		asset_id: String,
		author: NbtTranslateColor,
		height: i32,
		title: NbtTranslateColor,
		width: i32
	},
	"minecraft:pig_variant", PigVariant => {
		asset_id: String,
		model: Option<String>
	},
	"minecraft:trim_pattern", TrimPattern => {
		asset_id: String,
		decal: i8,
		description: NbtTranslateColor
	},
	"minecraft:wolf_sound_variant", WolfSoundVariant => {
		pant_sound: String,
		hurt_sound: String,
		growl_sound: String,
		whine_sound: String,
		death_sound: String,
		ambient_sound: String
	},
	"minecraft:wolf_variant", WolfVariant => {
		angry: String,
		tame: String,
		wild: String
	},
	"minecraft:worldgen/biome", Biome => {
		downfall: f32,
		has_precipitation: bool, // note that bools will be shown as "Byte" in JSON form of nbt
		temperature: f32,
		temperature_modifier: Option<String>
	}

);

#[cfg(test)]
mod test {
	use crate::protocol::game::info::registry::{BannerPattern, DimensionType, RegistryDataPacketInternal, RegistryEntry, RegistryType};
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};

	#[test]
	fn dimensiontype_asnbt() {
		let dim = DimensionType::default();
		let original = dim.to_nbt();
		let from_nbt = DimensionType::from_nbt(&original).unwrap();
		assert_eq!(dim, from_nbt);
		let second = from_nbt.to_nbt();
		assert_eq!(original, second);
	}

	#[test]
	fn serialize_and_back_again() {
		let dim = BannerPattern::default();
		let mut serializer = McSerializer::new();
		dim.mc_serialize(&mut serializer).unwrap();

		println!("{:?}", dim.to_nbt());

		let mut deserializer = McDeserializer::new(serializer.as_bytes());
		println!("Deserializing: {:?}", deserializer.data);
		println!("First byte: {}", deserializer.data[0]);
		let deserialized: BannerPattern = McDeserialize::mc_deserialize(&mut deserializer).unwrap();

		assert_eq!(dim, deserialized);
	}

	#[test]
	fn test_registry_internal() {
		let registry = RegistryDataPacketInternal {
			registry_id: "minecraft:dimension_type".to_string(),
			num_entries: 0.into(),
			entries: vec![],
		};

		let mut serializer = McSerializer::new();
		registry.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(serializer.as_bytes());
		let deserialized: RegistryDataPacketInternal = McDeserialize::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(registry, deserialized);
	}

	#[test]
	fn test_registry_components() {
		let dim = DimensionType::default();
		let entry = RegistryEntry {
			id: "minecraft:overworld".to_string(),
			is_present: true,
			data: Some(RegistryType::DimensionType(dim)),
		};

		let registry = RegistryDataPacketInternal {
			registry_id: "minecraft:dimension_type".to_string(),
			num_entries: 1.into(),
			entries: vec![entry.clone()],
		};

		let mut serializer = McSerializer::new();
		registry.mc_serialize(&mut serializer).unwrap();
		println!("Serialized: {:?}", serializer.as_bytes());
		let mut deserializer = McDeserializer::new(serializer.as_bytes());
		let deserialized: RegistryDataPacketInternal = McDeserialize::mc_deserialize(&mut deserializer).unwrap();
		assert_eq!(registry, deserialized);
		assert_eq!(deserialized.entries.len(), 1);
		assert_eq!(deserialized.entries[0].id, "minecraft:overworld");

		assert_eq!(entry, deserialized.entries[0]);

	}

	#[test]
	fn test_registry_entry() {
		let mut serializer = McSerializer::new();
		let dim = DimensionType::default();
		let entry = RegistryEntry {
			id: "minecraft:overworld".to_string(),
			is_present: true,
			data: Some(RegistryType::DimensionType(dim)),
		};

		entry.mc_serialize(&mut serializer).unwrap();
		let mut deserializer = McDeserializer::new(serializer.as_bytes());
		let deserialized = RegistryEntry::mc_deserialize(&mut deserializer, "minecraft:dimension_type".to_string()).unwrap();
		assert_eq!(entry, deserialized);
	}
}