#![allow(non_snake_case)]

//! Registry data structures for specific details about biomes, dimensions, datapacks, etc.
//! https://minecraft.wiki/w/Java_Edition_protocol/Registry_data

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;
use crate::registry_entry;
use sandstone_derive::{McDeserialize, McSerialize};

#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct RegistryDataPacket {
	pub registry_id: String,
	pub entries: PrefixedArray<RegistryEntry>
}

// todo: deserialize using id, but which one

#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct RegistryEntry {
	pub id: String,
	pub data: PrefixedOptional<NbtCompound>,
}

/// Define a registry data group sent by a RegistryDataPacket.
#[macro_export]
macro_rules! registry_entry {
	(
		$mc_name:literal, $lib_name:ident => {
			$( $field_name:ident : $field_type:ty ),*
		}
	) => {
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
				Self::from_nbt(&nbt)
			}
		}
		
		#[derive(Debug, Clone, PartialEq)]
		pub enum RegistryType {
			$lib_name($lib_name),
		}
		
		impl RegistryType {
			pub fn deserialize<'a>(deserializer: &'a mut McDeserializer, string: String) -> SerializingResult<'a, Self> {
				match string.as_str() {
					$mc_name => {
						let entry = $lib_name::mc_deserialize(deserializer)?;
						Ok(RegistryType::$lib_name(entry))
					},
					_ => Err(SerializingErr::UniqueFailure(format!("Unknown registry type: {}", string))),
				}
			}
		}
		
		impl McSerialize for RegistryType {
			fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
				match self {
					RegistryType::$lib_name(entry) => entry.mc_serialize(serializer),
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
	use crate::protocol::game::info::registry::DimensionType;

	#[test]
	fn dimensiontype_asnbt() {
		let dim = DimensionType::default();
		let original = dim.to_nbt();
		let from_nbt = DimensionType::from_nbt(&original).unwrap();
		assert_eq!(dim, from_nbt);
		let second = from_nbt.to_nbt();
		assert_eq!(original, second);
	}
}