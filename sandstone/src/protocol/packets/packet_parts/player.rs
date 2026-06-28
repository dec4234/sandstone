use crate::bitflag;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
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

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PositionVarInt {
	pub x: VarInt,
	pub y: VarInt,
	pub z: VarInt,
}

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
