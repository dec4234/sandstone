use crate::protocol::game::effects::particle::Particle;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::internal_types::IDSet;
use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BlockParticleAlternative {
	pub particle_id: VarInt,
	pub particle_data: Particle,
	pub scaling: f32,
	pub speed: f32,
	pub weight: VarInt
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockPredicate {
	pub blocks: PrefixedOptional<IDSet>,
	pub properties: PrefixedOptional<PrefixedArray<BlockProperty>>,
	pub nbt: PrefixedOptional<NbtCompound>,
}

impl McSerialize for BlockPredicate {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.blocks.mc_serialize(serializer)?;
		self.properties.mc_serialize(serializer)?;
		self.nbt.mc_serialize(serializer)?;
		Ok(())
	}
}

impl McDeserialize for BlockPredicate {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		Ok(Self {
			blocks: PrefixedOptional::mc_deserialize(deserializer)?,
			properties: PrefixedOptional::mc_deserialize(deserializer)?,
			nbt: PrefixedOptional::mc_deserialize(deserializer)?,
		})
	}
}

impl McDefault for BlockPredicate {
	fn mc_default() -> Self {
		Self {
			blocks: PrefixedOptional::new(None),
			properties: PrefixedOptional::new(None),
			nbt: PrefixedOptional::new(None),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockProperty {
	pub name: String,
	pub is_exact: bool,
	pub exact_value: Option<String>,
	pub min_value: Option<String>,
	pub max_value: Option<String>,
}

impl McSerialize for BlockProperty {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.name.mc_serialize(serializer)?;
		self.is_exact.mc_serialize(serializer)?;
		if self.is_exact {
			self.exact_value.mc_serialize(serializer)?;
		} else {
			self.min_value.mc_serialize(serializer)?;
			self.max_value.mc_serialize(serializer)?;
		}
		Ok(())
	}
}

impl McDeserialize for BlockProperty {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let name = String::mc_deserialize(deserializer)?;
		let is_exact = bool::mc_deserialize(deserializer)?;
		if is_exact {
			Ok(Self {
				name,
				is_exact,
				exact_value: Some(String::mc_deserialize(deserializer)?),
				min_value: None,
				max_value: None,
			})
		} else {
			Ok(Self {
				name,
				is_exact,
				exact_value: None,
				min_value: Some(String::mc_deserialize(deserializer)?),
				max_value: Some(String::mc_deserialize(deserializer)?),
			})
		}
	}
}

impl McDefault for BlockProperty {
	fn mc_default() -> Self {
		Self {
			name: String::mc_default(),
			is_exact: true,
			exact_value: Some(String::mc_default()),
			min_value: None,
			max_value: None,
		}
	}
}