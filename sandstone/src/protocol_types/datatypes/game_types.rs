//! Types found in game such as position, etc.

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
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

#[cfg(test)]
mod test {
	use crate::protocol_types::datatypes::game_types::Position;

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
}