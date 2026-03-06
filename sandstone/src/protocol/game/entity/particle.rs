use crate::protocol::game::info::inventory::slotdata::SlotData;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{
	McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult,
};
use crate::protocol_types::datatypes::game_types::Position;
use crate::protocol_types::datatypes::var_types::VarInt;

#[derive(Debug, Clone, PartialEq)]
pub enum VibrationSource {
	Block(Position),
	Entity(VarInt, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Particle {
	AngryVillager,                                          // 0
	Block(VarInt),                                          // 1
	BlockMarker(VarInt),                                    // 2
	Bubble,                                                 // 3
	Cloud,                                                  // 4
	CopperFireFlame,                                        // 5
	Crit,                                                   // 6
	DamageIndicator,                                        // 7
	DragonBreath(f32),                                      // 8
	DrippingLava,                                           // 9
	FallingLava,                                            // 10
	LandingLava,                                            // 11
	DrippingWater,                                          // 12
	FallingWater,                                           // 13
	Dust(i32, f32),                                         // 14
	DustColorTransition(i32, i32, f32),                     // 15
	Effect(i32, f32),                                       // 16
	ElderGuardian,                                          // 17
	EnchantedHit,                                           // 18
	Enchant,                                                // 19
	EndRod,                                                 // 20
	EntityEffect(i32),                                      // 21
	ExplosionEmitter,                                       // 22
	Explosion,                                              // 23
	Gust,                                                   // 24
	SmallGust,                                              // 25
	GustEmitterLarge,                                       // 26
	GustEmitterSmall,                                       // 27
	SonicBoom,                                              // 28
	FallingDust(VarInt),                                    // 29
	Firework,                                               // 30
	Fishing,                                                // 31
	Flame,                                                  // 32
	Infested,                                               // 33
	CherryLeaves,                                           // 34
	PaleOakLeaves,                                          // 35
	TintedLeaves(i32),                                      // 36
	SculkSoul,                                              // 37
	SculkCharge(f32),                                       // 38
	SculkChargePop,                                         // 39
	SoulFireFlame,                                          // 40
	Soul,                                                   // 41
	Flash(i32),                                             // 42
	HappyVillager,                                          // 43
	Composter,                                              // 44
	Heart,                                                  // 45
	InstantEffect(i32, f32),                                // 46
	Item(SlotData),                                         // 47
	Vibration(VibrationSource, VarInt),                     // 48
	Trail(f64, f64, f64, i32, VarInt),                      // 49
	ItemSlime,                                              // 50
	ItemCobweb,                                             // 51
	ItemSnowball,                                           // 52
	LargeSmoke,                                             // 53
	Lava,                                                   // 54
	Mycelium,                                               // 55
	Note,                                                   // 56
	Poof,                                                   // 57
	Portal,                                                 // 58
	Rain,                                                   // 59
	Smoke,                                                  // 60
	WhiteSmoke,                                             // 61
	Sneeze,                                                 // 62
	Spit,                                                   // 63
	SquidInk,                                               // 64
	SweepAttack,                                            // 65
	TotemOfUndying,                                         // 66
	Underwater,                                             // 67
	Splash,                                                 // 68
	Witch,                                                  // 69
	BubblePop,                                              // 70
	CurrentDown,                                            // 71
	BubbleColumnUp,                                         // 72
	Nautilus,                                               // 73
	Dolphin,                                                // 74
	CampfireCosySmoke,                                      // 75
	CampfireSignalSmoke,                                    // 76
	DrippingHoney,                                          // 77
	FallingHoney,                                           // 78
	LandingHoney,                                           // 79
	FallingNectar,                                          // 80
	FallingSporeBlossom,                                    // 81
	Ash,                                                    // 82
	CrimsonSpore,                                           // 83
	WarpedSpore,                                            // 84
	SporeBlossomAir,                                        // 85
	DrippingObsidianTear,                                   // 86
	FallingObsidianTear,                                    // 87
	LandingObsidianTear,                                    // 88
	ReversePortal,                                          // 89
	WhiteAsh,                                               // 90
	SmallFlame,                                             // 91
	Snowflake,                                              // 92
	DrippingDripstoneLava,                                  // 93
	FallingDripstoneLava,                                   // 94
	DrippingDripstoneWater,                                 // 95
	FallingDripstoneWater,                                  // 96
	GlowSquidInk,                                           // 97
	Glow,                                                   // 98
	WaxOn,                                                  // 99
	WaxOff,                                                 // 100
	ElectricSpark,                                          // 101
	Scrape,                                                 // 102
	Shriek(VarInt),                                         // 103
	EggCrack,                                               // 104
	DustPlume,                                              // 105
	TrialSpawnerDetection,                                  // 106
	TrialSpawnerDetectionOminous,                           // 107
	VaultConnection,                                        // 108
	DustPillar(VarInt),                                     // 109
	OminousSpawning,                                        // 110
	RaidOmen,                                               // 111
	TrialOmen,                                              // 112
	BlockCrumble(VarInt),                                   // 113
	Firefly,                                                // 114
}

impl McSerialize for Particle {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			Particle::AngryVillager => VarInt(0).mc_serialize(serializer),
			Particle::Block(state) => {
				VarInt(1).mc_serialize(serializer)?;
				state.mc_serialize(serializer)
			}
			Particle::BlockMarker(state) => {
				VarInt(2).mc_serialize(serializer)?;
				state.mc_serialize(serializer)
			}
			Particle::Bubble => VarInt(3).mc_serialize(serializer),
			Particle::Cloud => VarInt(4).mc_serialize(serializer),
			Particle::CopperFireFlame => VarInt(5).mc_serialize(serializer),
			Particle::Crit => VarInt(6).mc_serialize(serializer),
			Particle::DamageIndicator => VarInt(7).mc_serialize(serializer),
			Particle::DragonBreath(power) => {
				VarInt(8).mc_serialize(serializer)?;
				power.mc_serialize(serializer)
			}
			Particle::DrippingLava => VarInt(9).mc_serialize(serializer),
			Particle::FallingLava => VarInt(10).mc_serialize(serializer),
			Particle::LandingLava => VarInt(11).mc_serialize(serializer),
			Particle::DrippingWater => VarInt(12).mc_serialize(serializer),
			Particle::FallingWater => VarInt(13).mc_serialize(serializer),
			Particle::Dust(color, scale) => {
				VarInt(14).mc_serialize(serializer)?;
				color.mc_serialize(serializer)?;
				scale.mc_serialize(serializer)
			}
			Particle::DustColorTransition(from, to, scale) => {
				VarInt(15).mc_serialize(serializer)?;
				from.mc_serialize(serializer)?;
				to.mc_serialize(serializer)?;
				scale.mc_serialize(serializer)
			}
			Particle::Effect(color, power) => {
				VarInt(16).mc_serialize(serializer)?;
				color.mc_serialize(serializer)?;
				power.mc_serialize(serializer)
			}
			Particle::ElderGuardian => VarInt(17).mc_serialize(serializer),
			Particle::EnchantedHit => VarInt(18).mc_serialize(serializer),
			Particle::Enchant => VarInt(19).mc_serialize(serializer),
			Particle::EndRod => VarInt(20).mc_serialize(serializer),
			Particle::EntityEffect(color) => {
				VarInt(21).mc_serialize(serializer)?;
				color.mc_serialize(serializer)
			}
			Particle::ExplosionEmitter => VarInt(22).mc_serialize(serializer),
			Particle::Explosion => VarInt(23).mc_serialize(serializer),
			Particle::Gust => VarInt(24).mc_serialize(serializer),
			Particle::SmallGust => VarInt(25).mc_serialize(serializer),
			Particle::GustEmitterLarge => VarInt(26).mc_serialize(serializer),
			Particle::GustEmitterSmall => VarInt(27).mc_serialize(serializer),
			Particle::SonicBoom => VarInt(28).mc_serialize(serializer),
			Particle::FallingDust(state) => {
				VarInt(29).mc_serialize(serializer)?;
				state.mc_serialize(serializer)
			}
			Particle::Firework => VarInt(30).mc_serialize(serializer),
			Particle::Fishing => VarInt(31).mc_serialize(serializer),
			Particle::Flame => VarInt(32).mc_serialize(serializer),
			Particle::Infested => VarInt(33).mc_serialize(serializer),
			Particle::CherryLeaves => VarInt(34).mc_serialize(serializer),
			Particle::PaleOakLeaves => VarInt(35).mc_serialize(serializer),
			Particle::TintedLeaves(color) => {
				VarInt(36).mc_serialize(serializer)?;
				color.mc_serialize(serializer)
			}
			Particle::SculkSoul => VarInt(37).mc_serialize(serializer),
			Particle::SculkCharge(roll) => {
				VarInt(38).mc_serialize(serializer)?;
				roll.mc_serialize(serializer)
			}
			Particle::SculkChargePop => VarInt(39).mc_serialize(serializer),
			Particle::SoulFireFlame => VarInt(40).mc_serialize(serializer),
			Particle::Soul => VarInt(41).mc_serialize(serializer),
			Particle::Flash(color) => {
				VarInt(42).mc_serialize(serializer)?;
				color.mc_serialize(serializer)
			}
			Particle::HappyVillager => VarInt(43).mc_serialize(serializer),
			Particle::Composter => VarInt(44).mc_serialize(serializer),
			Particle::Heart => VarInt(45).mc_serialize(serializer),
			Particle::InstantEffect(color, power) => {
				VarInt(46).mc_serialize(serializer)?;
				color.mc_serialize(serializer)?;
				power.mc_serialize(serializer)
			}
			Particle::Item(slot) => {
				VarInt(47).mc_serialize(serializer)?;
				slot.mc_serialize(serializer)
			}
			Particle::Vibration(source, ticks) => {
				VarInt(48).mc_serialize(serializer)?;
				match source {
					VibrationSource::Block(pos) => {
						VarInt(0).mc_serialize(serializer)?;
						pos.mc_serialize(serializer)?;
					}
					VibrationSource::Entity(id, eye_height) => {
						VarInt(1).mc_serialize(serializer)?;
						id.mc_serialize(serializer)?;
						eye_height.mc_serialize(serializer)?;
					}
				}
				ticks.mc_serialize(serializer)
			}
			Particle::Trail(x, y, z, color, duration) => {
				VarInt(49).mc_serialize(serializer)?;
				x.mc_serialize(serializer)?;
				y.mc_serialize(serializer)?;
				z.mc_serialize(serializer)?;
				color.mc_serialize(serializer)?;
				duration.mc_serialize(serializer)
			}
			Particle::ItemSlime => VarInt(50).mc_serialize(serializer),
			Particle::ItemCobweb => VarInt(51).mc_serialize(serializer),
			Particle::ItemSnowball => VarInt(52).mc_serialize(serializer),
			Particle::LargeSmoke => VarInt(53).mc_serialize(serializer),
			Particle::Lava => VarInt(54).mc_serialize(serializer),
			Particle::Mycelium => VarInt(55).mc_serialize(serializer),
			Particle::Note => VarInt(56).mc_serialize(serializer),
			Particle::Poof => VarInt(57).mc_serialize(serializer),
			Particle::Portal => VarInt(58).mc_serialize(serializer),
			Particle::Rain => VarInt(59).mc_serialize(serializer),
			Particle::Smoke => VarInt(60).mc_serialize(serializer),
			Particle::WhiteSmoke => VarInt(61).mc_serialize(serializer),
			Particle::Sneeze => VarInt(62).mc_serialize(serializer),
			Particle::Spit => VarInt(63).mc_serialize(serializer),
			Particle::SquidInk => VarInt(64).mc_serialize(serializer),
			Particle::SweepAttack => VarInt(65).mc_serialize(serializer),
			Particle::TotemOfUndying => VarInt(66).mc_serialize(serializer),
			Particle::Underwater => VarInt(67).mc_serialize(serializer),
			Particle::Splash => VarInt(68).mc_serialize(serializer),
			Particle::Witch => VarInt(69).mc_serialize(serializer),
			Particle::BubblePop => VarInt(70).mc_serialize(serializer),
			Particle::CurrentDown => VarInt(71).mc_serialize(serializer),
			Particle::BubbleColumnUp => VarInt(72).mc_serialize(serializer),
			Particle::Nautilus => VarInt(73).mc_serialize(serializer),
			Particle::Dolphin => VarInt(74).mc_serialize(serializer),
			Particle::CampfireCosySmoke => VarInt(75).mc_serialize(serializer),
			Particle::CampfireSignalSmoke => VarInt(76).mc_serialize(serializer),
			Particle::DrippingHoney => VarInt(77).mc_serialize(serializer),
			Particle::FallingHoney => VarInt(78).mc_serialize(serializer),
			Particle::LandingHoney => VarInt(79).mc_serialize(serializer),
			Particle::FallingNectar => VarInt(80).mc_serialize(serializer),
			Particle::FallingSporeBlossom => VarInt(81).mc_serialize(serializer),
			Particle::Ash => VarInt(82).mc_serialize(serializer),
			Particle::CrimsonSpore => VarInt(83).mc_serialize(serializer),
			Particle::WarpedSpore => VarInt(84).mc_serialize(serializer),
			Particle::SporeBlossomAir => VarInt(85).mc_serialize(serializer),
			Particle::DrippingObsidianTear => VarInt(86).mc_serialize(serializer),
			Particle::FallingObsidianTear => VarInt(87).mc_serialize(serializer),
			Particle::LandingObsidianTear => VarInt(88).mc_serialize(serializer),
			Particle::ReversePortal => VarInt(89).mc_serialize(serializer),
			Particle::WhiteAsh => VarInt(90).mc_serialize(serializer),
			Particle::SmallFlame => VarInt(91).mc_serialize(serializer),
			Particle::Snowflake => VarInt(92).mc_serialize(serializer),
			Particle::DrippingDripstoneLava => VarInt(93).mc_serialize(serializer),
			Particle::FallingDripstoneLava => VarInt(94).mc_serialize(serializer),
			Particle::DrippingDripstoneWater => VarInt(95).mc_serialize(serializer),
			Particle::FallingDripstoneWater => VarInt(96).mc_serialize(serializer),
			Particle::GlowSquidInk => VarInt(97).mc_serialize(serializer),
			Particle::Glow => VarInt(98).mc_serialize(serializer),
			Particle::WaxOn => VarInt(99).mc_serialize(serializer),
			Particle::WaxOff => VarInt(100).mc_serialize(serializer),
			Particle::ElectricSpark => VarInt(101).mc_serialize(serializer),
			Particle::Scrape => VarInt(102).mc_serialize(serializer),
			Particle::Shriek(delay) => {
				VarInt(103).mc_serialize(serializer)?;
				delay.mc_serialize(serializer)
			}
			Particle::EggCrack => VarInt(104).mc_serialize(serializer),
			Particle::DustPlume => VarInt(105).mc_serialize(serializer),
			Particle::TrialSpawnerDetection => VarInt(106).mc_serialize(serializer),
			Particle::TrialSpawnerDetectionOminous => VarInt(107).mc_serialize(serializer),
			Particle::VaultConnection => VarInt(108).mc_serialize(serializer),
			Particle::DustPillar(state) => {
				VarInt(109).mc_serialize(serializer)?;
				state.mc_serialize(serializer)
			}
			Particle::OminousSpawning => VarInt(110).mc_serialize(serializer),
			Particle::RaidOmen => VarInt(111).mc_serialize(serializer),
			Particle::TrialOmen => VarInt(112).mc_serialize(serializer),
			Particle::BlockCrumble(state) => {
				VarInt(113).mc_serialize(serializer)?;
				state.mc_serialize(serializer)
			}
			Particle::Firefly => VarInt(114).mc_serialize(serializer),
		}
	}
}

impl McDeserialize for Particle {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let id = VarInt::mc_deserialize(deserializer)?.0;
		match id {
			0 => Ok(Particle::AngryVillager),
			1 => Ok(Particle::Block(VarInt::mc_deserialize(deserializer)?)),
			2 => Ok(Particle::BlockMarker(VarInt::mc_deserialize(deserializer)?)),
			3 => Ok(Particle::Bubble),
			4 => Ok(Particle::Cloud),
			5 => Ok(Particle::CopperFireFlame),
			6 => Ok(Particle::Crit),
			7 => Ok(Particle::DamageIndicator),
			8 => Ok(Particle::DragonBreath(f32::mc_deserialize(deserializer)?)),
			9 => Ok(Particle::DrippingLava),
			10 => Ok(Particle::FallingLava),
			11 => Ok(Particle::LandingLava),
			12 => Ok(Particle::DrippingWater),
			13 => Ok(Particle::FallingWater),
			14 => {
				let color = i32::mc_deserialize(deserializer)?;
				let scale = f32::mc_deserialize(deserializer)?;
				Ok(Particle::Dust(color, scale))
			}
			15 => {
				let from = i32::mc_deserialize(deserializer)?;
				let to = i32::mc_deserialize(deserializer)?;
				let scale = f32::mc_deserialize(deserializer)?;
				Ok(Particle::DustColorTransition(from, to, scale))
			}
			16 => {
				let color = i32::mc_deserialize(deserializer)?;
				let power = f32::mc_deserialize(deserializer)?;
				Ok(Particle::Effect(color, power))
			}
			17 => Ok(Particle::ElderGuardian),
			18 => Ok(Particle::EnchantedHit),
			19 => Ok(Particle::Enchant),
			20 => Ok(Particle::EndRod),
			21 => Ok(Particle::EntityEffect(i32::mc_deserialize(deserializer)?)),
			22 => Ok(Particle::ExplosionEmitter),
			23 => Ok(Particle::Explosion),
			24 => Ok(Particle::Gust),
			25 => Ok(Particle::SmallGust),
			26 => Ok(Particle::GustEmitterLarge),
			27 => Ok(Particle::GustEmitterSmall),
			28 => Ok(Particle::SonicBoom),
			29 => Ok(Particle::FallingDust(VarInt::mc_deserialize(deserializer)?)),
			30 => Ok(Particle::Firework),
			31 => Ok(Particle::Fishing),
			32 => Ok(Particle::Flame),
			33 => Ok(Particle::Infested),
			34 => Ok(Particle::CherryLeaves),
			35 => Ok(Particle::PaleOakLeaves),
			36 => Ok(Particle::TintedLeaves(i32::mc_deserialize(deserializer)?)),
			37 => Ok(Particle::SculkSoul),
			38 => Ok(Particle::SculkCharge(f32::mc_deserialize(deserializer)?)),
			39 => Ok(Particle::SculkChargePop),
			40 => Ok(Particle::SoulFireFlame),
			41 => Ok(Particle::Soul),
			42 => Ok(Particle::Flash(i32::mc_deserialize(deserializer)?)),
			43 => Ok(Particle::HappyVillager),
			44 => Ok(Particle::Composter),
			45 => Ok(Particle::Heart),
			46 => {
				let color = i32::mc_deserialize(deserializer)?;
				let power = f32::mc_deserialize(deserializer)?;
				Ok(Particle::InstantEffect(color, power))
			}
			47 => Ok(Particle::Item(SlotData::mc_deserialize(deserializer)?)),
			48 => {
				let source_type = VarInt::mc_deserialize(deserializer)?.0;
				let source = match source_type {
					0 => VibrationSource::Block(Position::mc_deserialize(deserializer)?),
					1 => {
						let entity_id = VarInt::mc_deserialize(deserializer)?;
						let eye_height = f32::mc_deserialize(deserializer)?;
						VibrationSource::Entity(entity_id, eye_height)
					}
					_ => return Err(SerializingErr::DeserializationError(
						format!("Invalid vibration source type: {}", source_type)
					)),
				};
				let ticks = VarInt::mc_deserialize(deserializer)?;
				Ok(Particle::Vibration(source, ticks))
			}
			49 => {
				let x = f64::mc_deserialize(deserializer)?;
				let y = f64::mc_deserialize(deserializer)?;
				let z = f64::mc_deserialize(deserializer)?;
				let color = i32::mc_deserialize(deserializer)?;
				let duration = VarInt::mc_deserialize(deserializer)?;
				Ok(Particle::Trail(x, y, z, color, duration))
			}
			50 => Ok(Particle::ItemSlime),
			51 => Ok(Particle::ItemCobweb),
			52 => Ok(Particle::ItemSnowball),
			53 => Ok(Particle::LargeSmoke),
			54 => Ok(Particle::Lava),
			55 => Ok(Particle::Mycelium),
			56 => Ok(Particle::Note),
			57 => Ok(Particle::Poof),
			58 => Ok(Particle::Portal),
			59 => Ok(Particle::Rain),
			60 => Ok(Particle::Smoke),
			61 => Ok(Particle::WhiteSmoke),
			62 => Ok(Particle::Sneeze),
			63 => Ok(Particle::Spit),
			64 => Ok(Particle::SquidInk),
			65 => Ok(Particle::SweepAttack),
			66 => Ok(Particle::TotemOfUndying),
			67 => Ok(Particle::Underwater),
			68 => Ok(Particle::Splash),
			69 => Ok(Particle::Witch),
			70 => Ok(Particle::BubblePop),
			71 => Ok(Particle::CurrentDown),
			72 => Ok(Particle::BubbleColumnUp),
			73 => Ok(Particle::Nautilus),
			74 => Ok(Particle::Dolphin),
			75 => Ok(Particle::CampfireCosySmoke),
			76 => Ok(Particle::CampfireSignalSmoke),
			77 => Ok(Particle::DrippingHoney),
			78 => Ok(Particle::FallingHoney),
			79 => Ok(Particle::LandingHoney),
			80 => Ok(Particle::FallingNectar),
			81 => Ok(Particle::FallingSporeBlossom),
			82 => Ok(Particle::Ash),
			83 => Ok(Particle::CrimsonSpore),
			84 => Ok(Particle::WarpedSpore),
			85 => Ok(Particle::SporeBlossomAir),
			86 => Ok(Particle::DrippingObsidianTear),
			87 => Ok(Particle::FallingObsidianTear),
			88 => Ok(Particle::LandingObsidianTear),
			89 => Ok(Particle::ReversePortal),
			90 => Ok(Particle::WhiteAsh),
			91 => Ok(Particle::SmallFlame),
			92 => Ok(Particle::Snowflake),
			93 => Ok(Particle::DrippingDripstoneLava),
			94 => Ok(Particle::FallingDripstoneLava),
			95 => Ok(Particle::DrippingDripstoneWater),
			96 => Ok(Particle::FallingDripstoneWater),
			97 => Ok(Particle::GlowSquidInk),
			98 => Ok(Particle::Glow),
			99 => Ok(Particle::WaxOn),
			100 => Ok(Particle::WaxOff),
			101 => Ok(Particle::ElectricSpark),
			102 => Ok(Particle::Scrape),
			103 => Ok(Particle::Shriek(VarInt::mc_deserialize(deserializer)?)),
			104 => Ok(Particle::EggCrack),
			105 => Ok(Particle::DustPlume),
			106 => Ok(Particle::TrialSpawnerDetection),
			107 => Ok(Particle::TrialSpawnerDetectionOminous),
			108 => Ok(Particle::VaultConnection),
			109 => Ok(Particle::DustPillar(VarInt::mc_deserialize(deserializer)?)),
			110 => Ok(Particle::OminousSpawning),
			111 => Ok(Particle::RaidOmen),
			112 => Ok(Particle::TrialOmen),
			113 => Ok(Particle::BlockCrumble(VarInt::mc_deserialize(deserializer)?)),
			114 => Ok(Particle::Firefly),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid particle type: {}", id))),
		}
	}
}