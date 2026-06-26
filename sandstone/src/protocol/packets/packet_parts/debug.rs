//! Debug subscription types. Used in certain packets to help with debugging the game.
//! https://minecraft.wiki/w/Java_Edition_protocol

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::game_types::Position;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

/// A debug subscription event: a debug subscription type followed by its data. The type tag and
/// the data are carried together by the tagged [DebugSubscriptionData] enum.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct DebugSubscriptionEvent {
	pub data: DebugSubscriptionData,
}

/// A debug subscription update: a debug subscription type followed by its optional data. The type
/// tag travels with the data inside [DebugSubscriptionData]; when absent the subscription carries
/// no value.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct DebugSubscriptionUpdate {
	pub data: PrefixedOptional<DebugSubscriptionData>,
}

/// The value of a debug subscription. The VarInt discriminant identifies the subscription type and
/// selects the associated data.
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum DebugSubscriptionData {
	DedicatedServerTickTime = 0,
	Bee(BeeDebugData) = 1,
	VillagerBrain(VillagerBrainDebugData) = 2,
	Breeze(BreezeDebugData) = 3,
	GoalSelector(GoalSelectorDebugData) = 4,
	EntityPath(EntityPathDebugData) = 5,
	EntityBlockIntersection(EntityBlockIntersectionType) = 6,
	BeeHive(BeeHiveDebugData) = 7,
	Poi(PoiDebugData) = 8,
	RedstoneWireOrientation(VarInt) = 9,
	VillageSection = 10,
	Raid(RaidDebugData) = 11,
	Structure(StructureDebugData) = 12,
	GameEventListener(VarInt) = 13,
	NeighborUpdate(Position) = 14,
	GameEvent(GameEventDebugData) = 15,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BeeDebugData {
	pub hive_position: PrefixedOptional<Position>,
	pub flower_position: PrefixedOptional<Position>,
	pub travel_ticks: VarInt,
	pub blacklisted_hives: PrefixedArray<Position>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct VillagerBrainDebugData {
	pub name: String,
	pub profession: String,
	pub xp: i32,
	pub health: f32,
	pub max_health: f32,
	pub inventory: String,
	pub wants_golem: bool,
	pub anger_level: i32,
	pub activities: PrefixedArray<String>,
	pub behaviors: PrefixedArray<String>,
	pub memories: PrefixedArray<String>,
	pub gossips: PrefixedArray<String>,
	pub pois: PrefixedArray<Position>,
	pub potential_pois: PrefixedArray<Position>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BreezeDebugData {
	pub attack_target: PrefixedOptional<VarInt>,
	pub jump_target: PrefixedOptional<Position>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct GoalSelectorDebugData {
	pub priority: VarInt,
	pub is_running: bool,
	pub name: String,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct EntityPathDebugData {
	pub reached: bool,
	pub next_block_index: i32,
	pub block_position: Position,
	pub nodes: PrefixedArray<DebugPathNode>,
	pub target_nodes: PrefixedArray<DebugPathNode>,
	pub open_set: PrefixedArray<DebugPathNode>,
	pub closed_set: PrefixedArray<DebugPathNode>,
	pub max_node_distance: f32,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum EntityBlockIntersectionType {
	InBlock = 0,
	InFluid = 1,
	InAir = 2,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BeeHiveDebugData {
	/// ID in the `minecraft:block` registry.
	pub typ: VarInt,
	pub occupant_count: VarInt,
	pub honey_level: VarInt,
	pub sedated: bool,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PoiDebugData {
	pub position: Position,
	/// ID in the `minecraft:point_of_interest_type` registry.
	pub typ: VarInt,
	pub free_ticket_count: VarInt,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct RaidDebugData {
	pub positions: PrefixedArray<Position>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct StructureDebugData {
	pub structures: PrefixedArray<DebugStructureInfo>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct GameEventDebugData {
	/// ID in the `minecraft:game_event` registry.
	pub event: VarInt,
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct DebugPathNode {
	pub x: i32,
	pub y: i32,
	pub z: i32,
	pub walked_distance: f32,
	pub cost_malus: f32,
	pub closed: bool,
	pub typ: DebugPathNodeType,
	pub f: f32,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum DebugPathNodeType {
	Blocked = 0,
	Open = 1,
	Walkable = 2,
	WalkableDoor = 3,
	Trapdoor = 4,
	PowderSnow = 5,
	DangerPowderSnow = 6,
	Fence = 7,
	Lava = 8,
	Water = 9,
	WaterBorder = 10,
	Rail = 11,
	UnpassableRail = 12,
	DangerFire = 13,
	DamageFire = 14,
	DangerOther = 15,
	DamageOther = 16,
	DoorOpen = 17,
	DoorWoodClosed = 18,
	DoorIronClosed = 19,
	Breach = 20,
	Leaves = 21,
	StickyHoney = 22,
	Cocoa = 23,
	DamageCautious = 24,
	DangerTrapdoor = 25,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct DebugStructureInfo {
	pub bounding_box_min: Position,
	pub bounding_box_max: Position,
	pub pieces: PrefixedArray<DebugStructurePiece>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct DebugStructurePiece {
	pub bounding_box_min: Position,
	pub bounding_box_max: Position,
	pub is_start: bool,
}

/// The type of a debug sample, determining how the accompanying `Prefixed Array of Long` is
/// interpreted.
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum DebugSampleType {
	/// Four tick-related metrics in nanoseconds: full tick time, server tick time, tasks time,
	/// and idle time.
	TickTime = 0,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct CustomReportDetail {
	pub title: String,
	pub description: String,
}
