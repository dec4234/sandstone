use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct StatisticAward {
	pub category: StatCategory,
	pub stat_id: StatID,
	pub value: VarInt,
}

/// Statistic categories defined in the `minecraft:stat_type` registry. Each category determines
/// which registry the associated statistic ID refers to (block, item, entity_type, or custom_stat).
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum StatCategory {
	Mined = 0,
	Crafted = 1,
	Used = 2,
	Broken = 3,
	PickedUp = 4,
	Dropped = 5,
	Killed = 6,
	KilledBy = 7,
	Custom = 8,
}

/// Custom statistic IDs defined in the `minecraft:custom_stat` registry. Used when the
/// [StatCategory] is [StatCategory::Custom].
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum StatID {
	LeaveGame = 0,
	PlayTime = 1,
	TotalWorldTime = 2,
	TimeSinceDeath = 3,
	TimeSinceRest = 4,
	SneakTime = 5,
	WalkOneCm = 6,
	CrouchOneCm = 7,
	SprintOneCm = 8,
	WalkOnWaterOneCm = 9,
	FallOneCm = 10,
	ClimbOneCm = 11,
	FlyOneCm = 12,
	WalkUnderWaterOneCm = 13,
	MinecartOneCm = 14,
	BoatOneCm = 15,
	PigOneCm = 16,
	HappyGhastOneCm = 17,
	HorseOneCm = 18,
	AviateOneCm = 19,
	SwimOneCm = 20,
	StriderOneCm = 21,
	Jump = 22,
	Drop = 23,
	DamageDealt = 24,
	DamageDealtAbsorbed = 25,
	DamageDealtResisted = 26,
	DamageTaken = 27,
	DamageBlockedByShield = 28,
	DamageAbsorbed = 29,
	DamageResisted = 30,
	Deaths = 31,
	MobKills = 32,
	AnimalsBred = 33,
	PlayerKills = 34,
	FishCaught = 35,
	TalkedToVillager = 36,
	TradedWithVillager = 37,
	EatCakeSlice = 38,
	FillCauldron = 39,
	UseCauldron = 40,
	CleanArmor = 41,
	CleanBanner = 42,
	CleanShulkerBox = 43,
	InteractWithBrewingstand = 44,
	InteractWithBeacon = 45,
	InspectDropper = 46,
	InspectHopper = 47,
	InspectDispenser = 48,
	PlayNoteblock = 49,
	TuneNoteblock = 50,
	PotFlower = 51,
	TriggerTrappedChest = 52,
	OpenEnderchest = 53,
	EnchantItem = 54,
	PlayRecord = 55,
	InteractWithFurnace = 56,
	InteractWithCraftingTable = 57,
	OpenChest = 58,
	SleepInBed = 59,
	OpenShulkerBox = 60,
	OpenBarrel = 61,
	InteractWithBlastFurnace = 62,
	InteractWithSmoker = 63,
	InteractWithLectern = 64,
	InteractWithCampfire = 65,
	InteractWithCartographyTable = 66,
	InteractWithLoom = 67,
	InteractWithStonecutter = 68,
	BellRing = 69,
	RaidTrigger = 70,
	RaidWin = 71,
	InteractWithAnvil = 72,
	InteractWithGrindstone = 73,
	TargetHit = 74,
	InteractWithSmithingTable = 75,
}