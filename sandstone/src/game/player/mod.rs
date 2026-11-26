use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::McDefault;

/// The Gamemode of a player, sent as a VarInt enum (0: Survival, 1: Creative, 2: Adventure, 3: Spectator).
///
/// `UNDEFINED` is only valid on the wire inside [PreviousGamemode]. It is written as VarInt -1 here.
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_(play)>
#[derive(McDefault, Debug, Clone, Copy, Hash, PartialEq)]
#[repr(i8)]
pub enum PlayerGamemode {
	UNDEFINED = -1,
	SURVIVAL = 0,
	CREATIVE = 1,
	ADVENTURE = 2,
	SPECTATOR = 3,
}

impl PlayerGamemode {
	fn from_id(id: i32) -> SerializingResult<'static, Self> {
		match id {
			-1 => Ok(PlayerGamemode::UNDEFINED),
			0 => Ok(PlayerGamemode::SURVIVAL),
			1 => Ok(PlayerGamemode::CREATIVE),
			2 => Ok(PlayerGamemode::ADVENTURE),
			3 => Ok(PlayerGamemode::SPECTATOR),
			_ => Err(SerializingErr::OutOfBounds(format!("Invalid gamemode id: {}", id))),
		}
	}
}

impl McSerialize for PlayerGamemode {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		VarInt(*self as i32).mc_serialize(serializer)
	}
}

impl McDeserialize for PlayerGamemode {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let id = VarInt::mc_deserialize(deserializer)?.0;
		PlayerGamemode::from_id(id)
	}
}

/// The "Previous Game mode" field of Login (play) and Respawn. It is a VarInt where 0 means
/// undefined, and any other value is the gamemode ID plus 1.
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_(play)>
#[derive(McDefault, Debug, Clone, Copy, Hash, PartialEq)]
pub struct PreviousGamemode(pub PlayerGamemode);

impl McSerialize for PreviousGamemode {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		VarInt(self.0 as i32 + 1).mc_serialize(serializer)
	}
}

impl McDeserialize for PreviousGamemode {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let id = VarInt::mc_deserialize(deserializer)?.0;
		Ok(PreviousGamemode(PlayerGamemode::from_id(id - 1)?))
	}
}

#[test]
fn test_gamemode_serialization() {
	// Wiki (Login (play)): Game mode is a VarInt enum. Values 0..=3 fit in one VarInt byte.
	let mut serializer = McSerializer::new();
	PlayerGamemode::SURVIVAL.mc_serialize(&mut serializer).unwrap();
	PlayerGamemode::CREATIVE.mc_serialize(&mut serializer).unwrap();
	PlayerGamemode::ADVENTURE.mc_serialize(&mut serializer).unwrap();
	PlayerGamemode::SPECTATOR.mc_serialize(&mut serializer).unwrap();
	let bytes = serializer.as_bytes();

	assert_eq!(bytes, vec![0, 1, 2, 3]);
}

/// The client reads Previous Game mode as "ID + 1, 0 = undefined". If we wrote the plain ID, the
/// client would show the wrong previous mode (and undefined would become a 5-byte -1).
#[test]
fn test_previous_gamemode_is_id_plus_one() {
	let mut serializer = McSerializer::new();
	for mode in [PlayerGamemode::UNDEFINED, PlayerGamemode::SURVIVAL, PlayerGamemode::CREATIVE, PlayerGamemode::ADVENTURE, PlayerGamemode::SPECTATOR] {
		PreviousGamemode(mode).mc_serialize(&mut serializer).unwrap();
	}
	let bytes = serializer.as_bytes().to_vec();
	assert_eq!(bytes, vec![0, 1, 2, 3, 4]);

	let mut deserializer = McDeserializer::new(&bytes);
	assert_eq!(PreviousGamemode::mc_deserialize(&mut deserializer).unwrap(), PreviousGamemode(PlayerGamemode::UNDEFINED));
	assert_eq!(PreviousGamemode::mc_deserialize(&mut deserializer).unwrap(), PreviousGamemode(PlayerGamemode::SURVIVAL));
	assert!(PreviousGamemode::mc_deserialize(&mut McDeserializer::new(&[5])).is_err());
}
