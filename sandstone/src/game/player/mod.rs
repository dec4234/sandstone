use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use sandstone_derive::McSerialize;

#[derive(McSerialize, Debug, Clone, Hash, PartialEq)]
#[repr(i8)]
pub enum PlayerGamemode {
	UNDEFINED = -1,
	SURVIVAL = 0,
	CREATIVE = 1,
	ADVENTURE = 2,
	SPECTATOR = 3,
}

impl McDeserialize for PlayerGamemode {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let i = i8::mc_deserialize(deserializer)?;

		match i {
			-1 => Ok(PlayerGamemode::UNDEFINED),
			0 => Ok(PlayerGamemode::SURVIVAL),
			1 => Ok(PlayerGamemode::CREATIVE),
			2 => Ok(PlayerGamemode::ADVENTURE),
			3 => Ok(PlayerGamemode::SPECTATOR),
			_ => Err(SerializingErr::InvalidEnumValue(i)),
		}
	}
}
