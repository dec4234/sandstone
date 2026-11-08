use crate::protocol::game::player::inventory::slotdata::SlotData;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::game_types::Position;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::VarIntEnum;

#[derive(VarIntEnum, Debug, Copy, Clone, PartialEq)]
#[repr(i32)]
pub enum VibrationSource {
	Block(Position) = 0,
	Entity(VarInt, f32) = 1,
}

#[derive(VarIntEnum, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum Particle {
	AngryVillager = 0,
	Block(VarInt) = 1,
	BlockMarker(VarInt) = 2,
	Bubble = 3,
	Cloud = 4,
	CopperFireFlame = 5,
	Crit = 6,
	DamageIndicator = 7,
	DragonBreath(f32) = 8,
	DrippingLava = 9,
	FallingLava = 10,
	LandingLava = 11,
	DrippingWater = 12,
	FallingWater = 13,
	Dust(i32, f32) = 14,
	DustColorTransition(i32, i32, f32) = 15,
	Effect(i32, f32) = 16,
	ElderGuardian = 17,
	EnchantedHit = 18,
	Enchant = 19,
	EndRod = 20,
	EntityEffect(i32) = 21,
	ExplosionEmitter = 22,
	Explosion = 23,
	Gust = 24,
	SmallGust = 25,
	GustEmitterLarge = 26,
	GustEmitterSmall = 27,
	SonicBoom = 28,
	FallingDust(VarInt) = 29,
	Firework = 30,
	Fishing = 31,
	Flame = 32,
	Infested = 33,
	CherryLeaves = 34,
	PaleOakLeaves = 35,
	TintedLeaves(i32) = 36,
	SculkSoul = 37,
	SculkCharge(f32) = 38,
	SculkChargePop = 39,
	SoulFireFlame = 40,
	Soul = 41,
	Flash(i32) = 42,
	HappyVillager = 43,
	Composter = 44,
	Heart = 45,
	InstantEffect(i32, f32) = 46,
	Item(SlotData) = 47,
	Vibration(VibrationSource, VarInt) = 48,
	Trail(f64, f64, f64, i32, VarInt) = 49,
	ItemSlime = 50,
	ItemCobweb = 51,
	ItemSnowball = 52,
	LargeSmoke = 53,
	Lava = 54,
	Mycelium = 55,
	Note = 56,
	Poof = 57,
	Portal = 58,
	Rain = 59,
	Smoke = 60,
	WhiteSmoke = 61,
	Sneeze = 62,
	Spit = 63,
	SquidInk = 64,
	SweepAttack = 65,
	TotemOfUndying = 66,
	Underwater = 67,
	Splash = 68,
	Witch = 69,
	BubblePop = 70,
	CurrentDown = 71,
	BubbleColumnUp = 72,
	Nautilus = 73,
	Dolphin = 74,
	CampfireCosySmoke = 75,
	CampfireSignalSmoke = 76,
	DrippingHoney = 77,
	FallingHoney = 78,
	LandingHoney = 79,
	FallingNectar = 80,
	FallingSporeBlossom = 81,
	Ash = 82,
	CrimsonSpore = 83,
	WarpedSpore = 84,
	SporeBlossomAir = 85,
	DrippingObsidianTear = 86,
	FallingObsidianTear = 87,
	LandingObsidianTear = 88,
	ReversePortal = 89,
	WhiteAsh = 90,
	SmallFlame = 91,
	Snowflake = 92,
	DrippingDripstoneLava = 93,
	FallingDripstoneLava = 94,
	DrippingDripstoneWater = 95,
	FallingDripstoneWater = 96,
	GlowSquidInk = 97,
	Glow = 98,
	WaxOn = 99,
	WaxOff = 100,
	ElectricSpark = 101,
	Scrape = 102,
	Shriek(VarInt) = 103,
	EggCrack = 104,
	DustPlume = 105,
	TrialSpawnerDetection = 106,
	TrialSpawnerDetectionOminous = 107,
	VaultConnection = 108,
	DustPillar(VarInt) = 109,
	OminousSpawning = 110,
	RaidOmen = 111,
	TrialOmen = 112,
	BlockCrumble(VarInt) = 113,
	Firefly = 114,
}

impl McDefault for Particle {
	fn mc_default() -> Self {
		Self::Cloud
	}
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
	fn particle_id_comes_first() {
		// Wiki (Particle): the VarInt particle type ID, then the fields for that type.
		assert_eq!(ser(&Particle::AngryVillager), vec![0]);
		assert_eq!(ser(&Particle::Cloud), vec![4]);
		assert_eq!(ser(&Particle::Firefly), vec![114]);
		assert_eq!(ser(&Particle::Block(VarInt(1))), vec![1, 1]);
	}

	#[test]
	fn dust_is_int_color_then_float_scale() {
		let mut expected = vec![14];
		expected.extend_from_slice(&0x00FF00i32.to_be_bytes());
		expected.extend_from_slice(&2.0f32.to_be_bytes());
		assert_eq!(ser(&Particle::Dust(0x00FF00, 2.0)), expected);
	}

	#[test]
	fn particles_with_data_round_trip() {
		let empty_slot = SlotData { item_count: VarInt(0), item_id: None, components_to_add: vec![], components_to_remove: vec![] };
		for p in [
			Particle::DustColorTransition(1, 2, 0.5),
			Particle::Effect(3, 1.0),
			Particle::TintedLeaves(0x123456),
			Particle::Item(empty_slot),
			Particle::Vibration(VibrationSource::Block(Position::new(1, 2, 3)), VarInt(40)),
			Particle::Vibration(VibrationSource::Entity(VarInt(12), 1.5), VarInt(40)),
			Particle::Trail(1.0, 2.0, 3.0, 0xFFFFFF, VarInt(20)),
			Particle::Shriek(VarInt(5)),
			Particle::BlockCrumble(VarInt(9)),
		] {
			round_trip(p);
		}
	}

	#[test]
	fn unknown_particle_id_is_rejected() {
		assert!(Particle::mc_deserialize(&mut McDeserializer::new(&[115])).is_err());
	}
}
