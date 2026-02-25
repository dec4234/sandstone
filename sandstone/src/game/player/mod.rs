use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use sandstone_derive::McDefault;

#[derive(McDefault, Debug, Clone, Hash, PartialEq)]
#[repr(i8)]
pub enum PlayerGamemode {
	UNDEFINED = -1,
	SURVIVAL = 0,
	CREATIVE = 1,
	ADVENTURE = 2,
	SPECTATOR = 3,
}

impl McSerialize for PlayerGamemode {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			PlayerGamemode::UNDEFINED => (-1i8).mc_serialize(serializer),
			PlayerGamemode::SURVIVAL => (0i8).mc_serialize(serializer),
			PlayerGamemode::CREATIVE => (1i8).mc_serialize(serializer),
			PlayerGamemode::ADVENTURE => (2i8).mc_serialize(serializer),
			PlayerGamemode::SPECTATOR => (3i8).mc_serialize(serializer),
		}
	}
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

#[test]
fn test_gamemode_serialization() {
	let mut serializer = McSerializer::new();
	PlayerGamemode::UNDEFINED.mc_serialize(&mut serializer).unwrap();
	PlayerGamemode::SURVIVAL.mc_serialize(&mut serializer).unwrap();
	PlayerGamemode::CREATIVE.mc_serialize(&mut serializer).unwrap();
	PlayerGamemode::ADVENTURE.mc_serialize(&mut serializer).unwrap();
	PlayerGamemode::SPECTATOR.mc_serialize(&mut serializer).unwrap();
	let bytes = serializer.as_bytes();

	assert_eq!(bytes, vec![-1i8 as u8, 0, 1, 2, 3]);
}
