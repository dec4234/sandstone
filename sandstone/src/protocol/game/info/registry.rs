#![allow(non_snake_case)]

//! Registry data structures for specific details about biomes, dimensions, datapacks, etc.
//! https://minecraft.wiki/w/Java_Edition_protocol/Registry_data

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::registry_entry;
use sandstone_derive::McSerialize;

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryDataPacketInternal {
	/// The registry type this data is for, e.g. "minecraft:dimension_type"
	pub registry_id: String, 
	pub num_entries: VarInt,
	pub entries: Vec<RegistryEntry>
}

impl McSerialize for RegistryDataPacketInternal {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.registry_id.mc_serialize(serializer)?;
		self.entries.mc_serialize(serializer)?;
		Ok(())
	}
}

impl McDeserialize for RegistryDataPacketInternal {
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

// todo: deserialize using id, but which one

#[derive(McSerialize, Debug, Clone, PartialEq)]
pub struct RegistryEntry {
	/// The ID of the registry entry, e.g. "minecraft:overworld"
	pub id: String,
	pub is_present: bool,
	pub data: Option<RegistryType>,
}

impl RegistryEntry {
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
			#[derive(Debug, Clone, PartialEq)]
			pub struct $lib_name {
				pub id: String,
				$(
					pub $field_name: $field_type,
				)*
			}
			
			impl $lib_name {
				pub fn new($($field_name: $field_type),*) -> Self {
					Self {
						id: $mc_name.to_string(),
						$($field_name,)*
					}
				}
				
				pub fn from_nbt(nbt: &NbtCompound) -> Result<Self, SerializingErr> {
					let id = nbt.root_name.clone().unwrap_or($mc_name.to_string());
					
					Ok(Self {
						id,
						$(
							$field_name: nbt.map.get(stringify!($field_name))
								.ok_or_else(|| SerializingErr::NbtMissingField(stringify!($field_name).to_string()))?
								.clone().into(),
						)*
					})
				}
				
				pub fn to_nbt(&self) -> NbtCompound {
					let mut nbt = NbtCompound::new(Some(self.id.clone()));
					
					$(
						nbt.add(stringify!($field_name), self.$field_name.clone());
					)*
					
					nbt
				}
			}
			
			impl McSerialize for $lib_name {
				fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
					self.to_nbt().mc_serialize(serializer)
				}
			}
			
			impl McDeserialize for $lib_name {
				fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
					let nbt = NbtCompound::mc_deserialize(deserializer)?;
					println!("{:?}", nbt);
					Self::from_nbt(&nbt)
				}
			}
		)*
		
		#[derive(Debug, Clone, PartialEq)]
		pub enum RegistryType {
			$($lib_name($lib_name)),*
		}
		
		impl RegistryType {
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


// https://minecraft.wiki/w/Java_Edition_protocol/Registry_data
registry_entry!(
	"minecraft:banner_pattern", BannerPattern => {
		asset_id: String,
		translation_key: String
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
	}
);

#[cfg(test)]
mod test {
	use crate::protocol::game::info::registry::{BannerPattern, DimensionType};
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
		let deserialized: BannerPattern = McDeserialize::mc_deserialize(&mut deserializer).unwrap();
		
		assert_eq!(dim, deserialized);
	}
}