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

// <https://minecraft.wiki/w/Java_Edition_protocol/Data_types#Teleport_Flags>
bitflag!(TeleportFlags: i32 {
	relative_x, relative_y, relative_z, relative_yaw, relative_pitch, relative_velocity_x, relative_velocity_y, relative_velocity_z, rotate_velocity
});

// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Information_(configuration)>
// Bit 7 appears unused, so it has no name.
bitflag!(DisplayedSkinParts: u8 {
	cape, jacket, left_sleeve, right_sleeve, left_pants_leg, right_pants_leg, hat
});

#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum WaypointData {
	Empty = 0,
	Vec3I(PositionVarInt) = 1,
	Chunk(ChunkPositionVarInt) = 2,
	Azimuth(f32) = 3,
}

/// # Position VarInt (Packet Part)
/// Represents a block position in the world as 3 VarInts
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct PositionVarInt {
	pub x: VarInt,
	pub y: VarInt,
	pub z: VarInt,
}

/// # Chunk Position VarInt (Packet Part)
/// Represents a chunk location in the world as 2 VarInts
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct ChunkPositionVarInt {
	pub x: VarInt,
	pub z: VarInt,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum WaypointOperation {
	Track = 0,
	Untrack = 1,
	Update = 2,
}

/// # Player Action Status (Packet Part)
/// Used to communicate a change in player status.
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Action>
///
/// The IDs are the order of `ServerboundPlayerActionPacket.Action` in the 26.3 server jar. 26.3
/// added `CHANGE_DESTROY_DIRECTION` (1) and `STAB` (8), so the other IDs are not the same as on
/// older wiki pages.
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum PlayerActionStatus {
	StartedDigging = 0,
	/// The player still digs the same block, but looks at another face of it.
	ChangeDiggingDirection = 1,
	CancelledDigging = 2,
	FinishedDigging = 3,
	DropItemStack = 4,
	DropItem = 5,
	ShootArrowOrFinishEating = 6,
	SwapItemInHand = 7,
	/// An attack with a spear.
	Stab = 8,
}

#[cfg(test)]
mod player_action_test {
    use super::PlayerActionStatus;
    use crate::protocol::serialization::{McSerialize, McSerializer};

    #[test]
	fn ids_match_the_26_3_jar() {
		// A 26.3 client sends 5 for the drop key and 7 for the swap-hands key.
		let id = |s: PlayerActionStatus| {
			let mut ser = McSerializer::new();
			s.mc_serialize(&mut ser).unwrap();
			ser.as_bytes()[0]
		};
		assert_eq!(id(PlayerActionStatus::FinishedDigging), 3);
		assert_eq!(id(PlayerActionStatus::DropItem), 5);
		assert_eq!(id(PlayerActionStatus::SwapItemInHand), 7);
		assert_eq!(id(PlayerActionStatus::Stab), 8);
	}
}

/// # Seen Advancement Action (Packet Part)
/// Action that occurs when interacting with Advancements tab
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Seen_Advancements>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum SeenAdvancementsAction {
	OpenedTab = 0,
	ClosedScreen = 1,
}

/// # Hand (Packet Part)
/// The hand involved in an item use event
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Use_Item>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum UseItemHand {
	MainHand = 0,
	OffHand = 1
}

/// # Chat Mode (Packet Part)
/// Which chat messages the client wants to receive
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Chat#Client_chat_mode>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum ChatMode {
	Enabled = 0,
	CommandsOnly = 1,
	Hidden = 2,
}

/// # Main Hand (Packet Part)
/// The main hand set in the client settings. The values are not the same as [UseItemHand].
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Information_(configuration)>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum MainHand {
	Left = 0,
	Right = 1,
}

/// # Particle Status (Packet Part)
/// The particle setting of the client
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Information_(configuration)>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum ParticleStatus {
	All = 0,
	Decreased = 1,
	Minimal = 2,
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
	fn displayed_skin_part_bits_match_wiki() {
		// Wiki: bit 0 cape, 1 jacket, 2 left sleeve, 3 right sleeve, 4 left pants leg,
		// 5 right pants leg, 6 hat. A wrong bit shows or hides the wrong part of the skin.
		let only = |i: usize| {
			let mut b = [false; 7];
			b[i] = true;
			DisplayedSkinParts::new(b[0], b[1], b[2], b[3], b[4], b[5], b[6])
		};
		for i in 0..7 {
			assert_eq!(ser(&only(i)), vec![1u8 << i], "flag index {i}");
		}
		let parts = DisplayedSkinParts::mc_deserialize(&mut McDeserializer::new(&[0x41])).unwrap();
		assert!(parts.cape() && parts.hat() && !parts.jacket());
		// the unused bit 7 must not be lost on a round trip
		let parts = DisplayedSkinParts::mc_deserialize(&mut McDeserializer::new(&[0x80])).unwrap();
		assert_eq!(ser(&parts), vec![0x80]);
	}

	#[test]
	fn client_information_enum_ids_match_wiki() {
		// Main hand is 0 = Left, 1 = Right. This is not the same as UseItemHand (0 = main hand).
		// If they are mixed up, the server shows the player with the wrong main hand.
		assert_eq!(ser(&MainHand::Left), vec![0]);
		assert_eq!(ser(&MainHand::Right), vec![1]);
		assert_eq!(ser(&ChatMode::Enabled), vec![0]);
		assert_eq!(ser(&ChatMode::CommandsOnly), vec![1]);
		assert_eq!(ser(&ChatMode::Hidden), vec![2]);
		assert_eq!(ser(&ParticleStatus::All), vec![0]);
		assert_eq!(ser(&ParticleStatus::Decreased), vec![1]);
		assert_eq!(ser(&ParticleStatus::Minimal), vec![2]);
		round_trip(ChatMode::Hidden);
		round_trip(MainHand::Right);
		round_trip(ParticleStatus::Minimal);
	}

	#[test]
	fn action_enum_ids_match_wiki() {
		assert_eq!(ser(&PlayerActionStatus::StartedDigging), vec![0]);
		// 26.3 IDs (see the PlayerActionStatus doc), not the older wiki IDs.
		assert_eq!(ser(&PlayerActionStatus::SwapItemInHand), vec![7]);
		assert_eq!(ser(&WaypointOperation::Update), vec![2]);
		assert_eq!(ser(&SeenAdvancementsAction::ClosedScreen), vec![1]);
		assert_eq!(ser(&UseItemHand::OffHand), vec![1]);
		round_trip(PlayerActionStatus::ShootArrowOrFinishEating);
	}

	#[test]
	fn unknown_action_status_is_rejected() {
		// 8 (Stab) is the highest ID in 26.3, so 9 is the first unknown ID.
		assert!(PlayerActionStatus::mc_deserialize(&mut McDeserializer::new(&[9])).is_err());
	}
}
