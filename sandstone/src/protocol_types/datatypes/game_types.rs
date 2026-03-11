//! Types found in game such as position, etc.

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarLong;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, TypeEnum};

/// A Minecraft position, internally represented as a 64-bit integer.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct Position {
	data: u64
}

impl Position {
	pub fn new(x: i64, y: i64, z: i64) -> Self {
		let data: u64 = (((x & 0x3FFFFFF) << 38) | ((z & 0x3FFFFFF) << 12) | (y & 0xFFF)) as u64;
		Self {
			data
		}
	}

	fn sign_extend(value: u64, bits: u32) -> i64 {
		let shift = 64 - bits;
		((value << shift) as i64) >> shift
	}

	pub fn x(&self) -> i64 {
		let raw = (self.data >> 38) & 0x3FFFFFF;
		Self::sign_extend(raw, 26)
	}

	pub fn y(&self) -> i64 {
		let raw = self.data & 0xFFF;
		Self::sign_extend(raw, 12)
	}

	pub fn z(&self) -> i64 {
		let raw = (self.data >> 12) & 0x3FFFFFF;
		Self::sign_extend(raw, 26)
	}
}

#[derive(TypeEnum, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
#[type_enum(u8)]
pub enum GameDifficulty {
	Peaceful = 0,
	Easy = 1,
	Normal = 2,
	Hard = 3
}

impl McDefault for GameDifficulty {
	fn mc_default() -> Self {
		GameDifficulty::Normal
	}
}

#[derive(McDefault, TypeEnum, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
#[type_enum(u8)]
pub enum EquipmentSlot {
	MainHand = 0,
	OffHand = 1,
	Boots = 2,
	Leggigns = 3,
	Chestplate = 4,
	Helmet = 5,
	Body = 6,
	Saddle = 7,
}

#[derive(McDefault, TypeEnum, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
#[type_enum(i32)]
pub enum WorldEventType {
	DispenserDispenses = 1000,
	DispenserFailsToDispense = 1001,
	DispenserShoots = 1002,
	FireworkShot = 1004,
	FireExtinguished = 1009,
	PlayRecord = 1010,
	StopRecord = 1011,
	GhastWarns = 1015,
	GhastShoots = 1016,
	EnderDragonShoots = 1017,
	BlazeShoots = 1018,
	ZombieAttacksWoodenDoor = 1019,
	ZombieAttacksIronDoor = 1020,
	ZombieBreaksWoodenDoor = 1021,
	WitherBreaksBlock = 1022,
	WitherSpawned = 1023,
	WitherShoots = 1024,
	BatTakesOff = 1025,
	ZombieInfects = 1026,
	ZombieVillagerConverted = 1027,
	EnderDragonDies = 1028,
	AnvilDestroyed = 1029,
	AnvilUsed = 1030,
	AnvilLands = 1031,
	PortalTravel = 1032,
	ChorusFlowerGrows = 1033,
	ChorusFlowerDies = 1034,
	BrewingStandBrews = 1035,
	EndPortalCreated = 1038,
	PhantomBites = 1039,
	ZombieConvertsToDrowned = 1040,
	HuskConvertsToZombie = 1041,
	GrindstoneUsed = 1042,
	BookPageTurned = 1043,
	SmithingTableUsed = 1044,
	PointedDripstoneLanding = 1045,
	LavaDrippingOnCauldron = 1046,
	WaterDrippingOnCauldron = 1047,
	SkeletonConvertsToStray = 1048,
	CrafterSuccessfullyCrafts = 1049,
	CrafterFailsToCraft = 1050,
	ComposterComposts = 1500,
	LavaConvertsBlock = 1501,
	RedstoneTorchBurnsOut = 1502,
	EnderEyePlaced = 1503,
	FluidDripsFromDripstone = 1504,
	BoneMealParticles = 1505,
	DispenserSmoke = 2000,
	BlockBreak = 2001,
	SplashPotion = 2002,
	EyeOfEnderBreak = 2003,
	SpawnerSpawnsMob = 2004,
	DragonBreath = 2006,
	InstantSplashPotion = 2007,
	EnderDragonDestroysBlock = 2008,
	WetSpongeVaporizes = 2009,
	CrafterSmoke = 2010,
	BeeFertilizesPlant = 2011,
	TurtleEggPlaced = 2012,
	SmashAttack = 2013,
	EndGatewaySpawns = 3000,
	EnderDragonResurrected = 3001,
	ElectricSpark = 3002,
	CopperApplyWax = 3003,
	CopperRemoveWax = 3004,
	CopperScrapeOxidation = 3005,
	SculkCharge = 3006,
	SculkShriekerShriek = 3007,
	BlockFinishedBrushing = 3008,
	SnifferEggCracks = 3009,
	TrialSpawnerSpawnsMob = 3011,
	TrialSpawnerSpawnsMobAtLocation = 3012,
	TrialSpawnerDetectsPlayer = 3013,
	TrialSpawnerEjectsItem = 3014,
	VaultActivates = 3015,
	VaultDeactivates = 3016,
	VaultEjectsItem = 3017,
	CobwebWeaved = 3018,
	OminousTrialSpawnerDetectsPlayer = 3019,
	TrialSpawnerTurnsOminous = 3020,
	OminousItemSpawnerSpawnsItem = 3021,
}

/// Packed i64 for coordinates of the affected chunk.
///
/// 22 bits for x, z and 20 bits for y
#[derive(McDefault, McSerialize, McDeserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct ChunkSectionPosition {
	data: i64
}

impl ChunkSectionPosition {
	pub fn new(section_x: i32, section_y: i32, section_z: i32) -> Self {
		let data = ((section_x as i64 & 0x3FFFFF) << 42)
			| (section_y as i64 & 0xFFFFF)
			| ((section_z as i64 & 0x3FFFFF) << 20);
		Self { data }
	}

	/// Chunk X coordinate
	pub fn section_x(&self) -> i32 {
		(self.data >> 42) as i32
	}

	/// Chunk Y coordinate
	pub fn section_y(&self) -> i32 {
		((self.data << 44) >> 44) as i32
	}

	/// Chunk Z coordinate
	pub fn section_z(&self) -> i32 {
		((self.data << 22) >> 42) as i32
	}
}

/// Packed VarLong of block state id, and local x, y, z coords
#[derive(McDefault, McSerialize, McDeserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct SectionBlockEntry {
	data: VarLong
}

impl SectionBlockEntry {
	pub fn new(block_state_id: i32, local_x: u8, local_y: u8, local_z: u8) -> Self {
		let value = ((block_state_id as i64) << 12)
			| ((local_x as i64 & 0xF) << 8)
			| ((local_z as i64 & 0xF) << 4)
			| (local_y as i64 & 0xF);
		Self { data: VarLong(value) }
	}

	pub fn block_state_id(&self) -> i32 {
		(self.data.0 >> 12) as i32
	}

	pub fn local_x(&self) -> u8 {
		((self.data.0 >> 8) & 0xF) as u8
	}

	pub fn local_y(&self) -> u8 {
		(self.data.0 & 0xF) as u8
	}

	pub fn local_z(&self) -> u8 {
		((self.data.0 >> 4) & 0xF) as u8
	}
}

#[derive(TypeEnum, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
#[type_enum(i32)]
pub enum SmokeDirection {
	Down = 0,
	Up = 1,
	North = 2,
	South = 3,
	West = 4,
	East = 5,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, PartialOrd, PartialEq, Clone)]
pub struct SourcePosition {
	x: f64,
	y: f64,
	z: f64
}

#[cfg(test)]
mod test {
	use crate::protocol_types::datatypes::game_types::{ChunkSectionPosition, Position, SectionBlockEntry};

	#[test]
	fn test_position() {
		let pos = Position::new(1, 2, 3);
		assert_eq!(pos.x(), 1);
		assert_eq!(pos.y(), 2);
		assert_eq!(pos.z(), 3);

		let pos = Position::new(-1, -2, -3);
		assert_eq!(pos.x(), -1);
		assert_eq!(pos.y(), -2);
		assert_eq!(pos.z(), -3);
	}

	#[test]
	fn test_chunk_section_position() {
		let pos = ChunkSectionPosition::new(3, 5, 7);
		assert_eq!(pos.section_x(), 3);
		assert_eq!(pos.section_y(), 5);
		assert_eq!(pos.section_z(), 7);
	}

	#[test]
	fn test_chunk_section_position_negative() {
		let pos = ChunkSectionPosition::new(-1, -4, -10);
		assert_eq!(pos.section_x(), -1);
		assert_eq!(pos.section_y(), -4);
		assert_eq!(pos.section_z(), -10);
	}

	#[test]
	fn test_chunk_section_position_zero() {
		let pos = ChunkSectionPosition::new(0, 0, 0);
		assert_eq!(pos.section_x(), 0);
		assert_eq!(pos.section_y(), 0);
		assert_eq!(pos.section_z(), 0);
	}

	#[test]
	fn test_section_block_entry() {
		let entry = SectionBlockEntry::new(42, 3, 7, 12);
		assert_eq!(entry.block_state_id(), 42);
		assert_eq!(entry.local_x(), 3);
		assert_eq!(entry.local_y(), 7);
		assert_eq!(entry.local_z(), 12);
	}

	#[test]
	fn test_section_block_entry_zero() {
		let entry = SectionBlockEntry::new(0, 0, 0, 0);
		assert_eq!(entry.block_state_id(), 0);
		assert_eq!(entry.local_x(), 0);
		assert_eq!(entry.local_y(), 0);
		assert_eq!(entry.local_z(), 0);
	}

	#[test]
	fn test_section_block_entry_max_local() {
		let entry = SectionBlockEntry::new(1, 15, 15, 15);
		assert_eq!(entry.block_state_id(), 1);
		assert_eq!(entry.local_x(), 15);
		assert_eq!(entry.local_y(), 15);
		assert_eq!(entry.local_z(), 15);
	}
}