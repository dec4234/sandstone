use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, VarIntEnum};

pub mod player_action;

#[derive(McDefault, VarIntEnum, Debug, Clone, PartialEq)]
pub enum ClientStatusAction {
	PerformRespawn = 0,
	RequestStats = 1,
}

/// Data kept after a respawn.
///
/// [Doc link](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Respawn)
#[derive(McDefault, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RespawnKeptData {
	pub keep_attributes: bool,
	pub keep_metadata: bool,
}

impl McSerialize for RespawnKeptData {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let mut flags = 0u8;

		flags |= self.keep_attributes as u8;
		flags |= (self.keep_metadata as u8) << 1;

		flags.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for RespawnKeptData {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let flags = u8::mc_deserialize(deserializer)?;

		Ok(Self {
			keep_attributes: (flags & 1) != 0,
			keep_metadata: (flags & 2) != 0,
		})
	}
}