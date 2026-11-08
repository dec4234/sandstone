use crate::bitflag;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

bitflag!(TeleportFlags: i32 {
	relative_x, relative_y, relative_z, relative_yaw, relative_pitch, relative_velocity_x, relative_velocity_y, relative_velocity_z, rotate_velocity
});

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum WaypointData {
	Empty = 0,
	Vec3I(PositionVarInt) = 1,
	Chunk(ChunkPositionVarInt) = 2,
	Azimuth(f32) = 3,
}

/// # Position VarInt (Packet Part)
/// Represents a block position in the world as 3 VarInts
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PositionVarInt {
	pub x: VarInt,
	pub y: VarInt,
	pub z: VarInt,
}

/// # Chunk Position VarInt (Packet Part)
/// Represents a chunk location in the world as 2 VarInts
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ChunkPositionVarInt {
	pub x: VarInt,
	pub z: VarInt,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum WaypointOperation {
	Track = 0,
	Untrack = 1,
	Update = 2,
}

/// # Player Action Status (Packet Part)
/// Used to communicate a change in player status.
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Action
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum PlayerActionStatus {
	StartedDigging = 0,
	CancelledDigging = 1,
	FinishedDigging = 2,
	DropItemStack = 3,
	DropItem = 4,
	ShootArrowOrFinishEating = 5,
	SwapItemInHand = 6,
}

/// # Seen Advancement Action (Packet Part)
/// Action that occurs when interacting with Advancements tab
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Seen_Advancements
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum SeenAdvancementsAction {
	OpenedTab = 0,
	ClosedScreen = 1,
}

/// # Hand (Packet Part)
/// The hand involved in an item use event
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Use_Item
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum UseItemHand {
	MainHand = 0,
	OffHand = 1
}

#[cfg(test)]
mod test {
	use super::*;

	fn ser<T: McSerialize>(v: &T) -> Vec<u8> {
		let mut s = McSerializer::new();
		v.mc_serialize(&mut s).unwrap();
		s.as_bytes().to_vec()
	}

	fn round_trip<T: McSerialize + McDeserialize + PartialEq + std::fmt::Debug>(v: T) {
		let bytes = ser(&v);
		let mut d = McDeserializer::new(&bytes);
		assert_eq!(T::mc_deserialize(&mut d).unwrap(), v);
		assert!(d.is_at_end(), "all bytes must be used");
	}

	#[test]
	fn teleport_flag_bits_match_wiki() {
		// Wiki (Teleport Flags): an Int where bit 0 is X, 1 Y, 2 Z, 3 yaw, 4 pitch, 5..7 velocity
		// X/Y/Z and 8 rotate velocity. A wrong bit makes the client move the player wrongly.
		let only = |i: usize| {
			let mut b = [false; 9];
			b[i] = true;
			TeleportFlags::new(b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8])
		};
		for i in 0..9 {
			assert_eq!(ser(&only(i)), (1i32 << i).to_be_bytes().to_vec(), "flag index {i}");
		}
		let f = TeleportFlags::mc_deserialize(&mut McDeserializer::new(&0x0108i32.to_be_bytes())).unwrap();
		assert!(f.relative_yaw() && f.rotate_velocity() && !f.relative_x());
	}

	#[test]
	fn waypoint_data_ids_and_layout() {
		let vec3 = WaypointData::Vec3I(PositionVarInt { x: VarInt(1), y: VarInt(2), z: VarInt(3) });
		assert_eq!(ser(&vec3), vec![1, 1, 2, 3]);
		round_trip(vec3);
		let chunk = WaypointData::Chunk(ChunkPositionVarInt { x: VarInt(-1), z: VarInt(4) });
		assert_eq!(ser(&chunk)[0], 2);
		round_trip(chunk);
		round_trip(WaypointData::Azimuth(1.25));
		assert_eq!(ser(&WaypointData::Empty), vec![0]);
	}

	#[test]
	fn action_enum_ids_match_wiki() {
		assert_eq!(ser(&PlayerActionStatus::StartedDigging), vec![0]);
		assert_eq!(ser(&PlayerActionStatus::SwapItemInHand), vec![6]);
		assert_eq!(ser(&WaypointOperation::Update), vec![2]);
		assert_eq!(ser(&SeenAdvancementsAction::ClosedScreen), vec![1]);
		assert_eq!(ser(&UseItemHand::OffHand), vec![1]);
		round_trip(PlayerActionStatus::ShootArrowOrFinishEating);
	}

	#[test]
	fn unknown_action_status_is_rejected() {
		assert!(PlayerActionStatus::mc_deserialize(&mut McDeserializer::new(&[7])).is_err());
	}
}
