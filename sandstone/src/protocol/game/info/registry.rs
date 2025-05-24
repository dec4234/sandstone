//! https://gist.github.com/Mansitoh/e6c5cf8bbf17e9faf4e4e75bb3f4789d

use crate::registry_entry;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::serializer_error::SerializingErr;
use sandstone_derive::{McDeserialize, McSerialize};
use crate::protocol::serialization::serializer_types::PrefixedOptional;
use crate::protocol_types::datatypes::nbt::nbt::{NbtCompound, NbtTag};

#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct RegistryEntry {
	pub id: String,
	pub data: PrefixedOptional<NbtCompound>,
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
			
			/*pub fn from_nbt(nbt: &NbtCompound) -> Result<Self, SerializingErr> {
				let id = nbt.get_string("id")?;
				
				Ok(Self {
					id: id,
					$(
						$field_name: nbt[stringify!($field_name)]?,
					)*
				})
			}*/
			
			pub fn to_nbt(&self) -> NbtCompound {
				let mut nbt = NbtCompound::new(Some($mc_name));
				
				$(
					nbt.add(stringify!($field_name).to_string(), self.$field_name);
				)*
				
				nbt
			}
		}
		
		#[derive(Debug, Clone, PartialEq)]
		pub enum RegistryType {
			$lib_name($lib_name),
		}
		
		impl McSerialize for RegistryType {
			fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
				match self {
					RegistryType::$lib_name(entry) => {
						entry.to_nbt().mc_serialize(serializer)?;
					}
				}
				Ok(())
			}
		}
	};
}


// https://minecraft.wiki/w/Java_Edition_protocol/Registry_data
registry_entry!(
	"minecraft:dimension_type", DimensionType => {
		fixed_time: Option<i64>
	}
);