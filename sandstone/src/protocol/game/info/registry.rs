//! Registry data structures for specific details about 

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol_types::datatypes::nbt::nbt::NbtTag;
use crate::registry_entry;
use sandstone_derive::{McDeserialize, McSerialize};

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryDataPacket {
	pub id: String,
	pub entries: PrefixedArray<RegistryEntry>
}

// todo deserialization

#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct RegistryEntry {
	pub id: String,
	pub data: PrefixedOptional<NbtTag>,
}

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
		}
		
		impl McSerialize for $lib_name {
			fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
				let mut v: Vec<RegistryEntry> = Vec::new();
				
				$(
					let entry = RegistryEntry {
						id: self.id.clone(),
						data: PrefixedOptional::new(
							registry_entry_optional!($field_type, self.$field_name)
						),
					};
				
					v.push(entry);
				)*
				
				let entries = PrefixedArray::new(v);
				entries.mc_serialize(serializer)?;
				
				Ok(())
			}
		}
		
		#[derive(Debug, Clone, PartialEq)]
		pub enum RegistryType {
			$lib_name($lib_name),
		}
	};
}

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

// https://gist.github.com/Mansitoh/e6c5cf8bbf17e9faf4e4e75bb3f4789d
impl Default for DimensionType {
	fn default() -> Self {
		todo!()
	}
}