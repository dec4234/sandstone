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
	pub z: VarInt
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ChunkPositionVarInt {
	pub x: VarInt,
	pub z: VarInt
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum WaypointOperation {
	Track = 0,
	Untrack = 1,
	Update = 2,
}