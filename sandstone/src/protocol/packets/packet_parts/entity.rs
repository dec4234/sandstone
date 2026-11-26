use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::internal_types::{Angle, TripleDouble};
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

/// The status code sent in the Entity Event packet, represented as a byte.
///
/// Each code links to the entity (or entities) it applies to. The meaning of a
/// given code differs per entity, so variants are suffixed with their numeric
/// code to keep them unambiguous.
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Entity_statuses>
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
#[repr(i8)]
pub enum EntityStatusEnum {
	/// Arrow
	Arrow0 = 0,
	/// Minecart Spawner, Rabbit
	MinecartSpawnerRabbit1 = 1,
	/// None
	Unused2 = 2,
	/// Egg, Living Entity, Snowball
	EggLivingEntitySnowball3 = 3,
	/// Evoker Fangs, Hoglin, Iron Golem, Ravager, Warden, Zoglin
	EvokerFangsHoglinIronGolemRavagerWardenZoglin4 = 4,
	/// None
	Unused5 = 5,
	/// Abstract Horse, Tameable Animal
	AbstractHorseTameableAnimal6 = 6,
	/// Abstract Horse, Tameable Animal
	AbstractHorseTameableAnimal7 = 7,
	/// Wolf
	Wolf8 = 8,
	/// Player
	Player9 = 9,
	/// Minecart TNT, Sheep
	MinecartTntSheep10 = 10,
	/// Iron Golem
	IronGolem11 = 11,
	/// Villager
	Villager12 = 12,
	/// Villager
	Villager13 = 13,
	/// Villager
	Villager14 = 14,
	/// Witch
	Witch15 = 15,
	/// Zombie Villager
	ZombieVillager16 = 16,
	/// Firework Rocket
	FireworkRocket17 = 17,
	/// Allay, Animal
	AllayAnimal18 = 18,
	/// Squid
	Squid19 = 19,
	/// Mob
	Mob20 = 20,
	/// Guardian
	Guardian21 = 21,
	/// Player
	Player22 = 22,
	/// Player
	Player23 = 23,
	/// Player
	Player24 = 24,
	/// Player
	Player25 = 25,
	/// Player
	Player26 = 26,
	/// Player
	Player27 = 27,
	/// Player
	Player28 = 28,
	/// Living Entity
	LivingEntity29 = 29,
	/// Living Entity
	LivingEntity30 = 30,
	/// Fishing Hook
	FishingHook31 = 31,
	/// Armor Stand
	ArmorStand32 = 32,
	/// None
	Unused33 = 33,
	/// Iron Golem
	IronGolem34 = 34,
	/// Living Entity
	LivingEntity35 = 35,
	/// None
	Unused36 = 36,
	/// None
	Unused37 = 37,
	/// Dolphin
	Dolphin38 = 38,
	/// Ravager
	Ravager39 = 39,
	/// Ocelot
	Ocelot40 = 40,
	/// Ocelot
	Ocelot41 = 41,
	/// Villager
	Villager42 = 42,
	/// Player
	Player43 = 43,
	/// None
	Unused44 = 44,
	/// Fox
	Fox45 = 45,
	/// Living Entity
	LivingEntity46 = 46,
	/// Living Entity
	LivingEntity47 = 47,
	/// Living Entity
	LivingEntity48 = 48,
	/// Living Entity
	LivingEntity49 = 49,
	/// Living Entity
	LivingEntity50 = 50,
	/// Living Entity
	LivingEntity51 = 51,
	/// Living Entity
	LivingEntity52 = 52,
	/// Entity
	Entity53 = 53,
	/// Living Entity
	LivingEntity54 = 54,
	/// Living Entity
	LivingEntity55 = 55,
	/// Wolf
	Wolf56 = 56,
	/// None
	Unused57 = 57,
	/// Goat
	Goat58 = 58,
	/// Goat
	Goat59 = 59,
	/// Living Entity
	LivingEntity60 = 60,
	/// Warden
	Warden61 = 61,
	/// Warden
	Warden62 = 62,
	/// Sniffer
	Sniffer63 = 63,
}

impl McDefault for EntityStatusEnum {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		Self::Arrow0
	}
}

impl McSerialize for EntityStatusEnum {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		(*self as i8).mc_serialize(serializer)
	}
}

impl McDeserialize for EntityStatusEnum {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let i = i8::mc_deserialize(deserializer)?;

		match i {
			0 => Ok(EntityStatusEnum::Arrow0),
			1 => Ok(EntityStatusEnum::MinecartSpawnerRabbit1),
			2 => Ok(EntityStatusEnum::Unused2),
			3 => Ok(EntityStatusEnum::EggLivingEntitySnowball3),
			4 => Ok(EntityStatusEnum::EvokerFangsHoglinIronGolemRavagerWardenZoglin4),
			5 => Ok(EntityStatusEnum::Unused5),
			6 => Ok(EntityStatusEnum::AbstractHorseTameableAnimal6),
			7 => Ok(EntityStatusEnum::AbstractHorseTameableAnimal7),
			8 => Ok(EntityStatusEnum::Wolf8),
			9 => Ok(EntityStatusEnum::Player9),
			10 => Ok(EntityStatusEnum::MinecartTntSheep10),
			11 => Ok(EntityStatusEnum::IronGolem11),
			12 => Ok(EntityStatusEnum::Villager12),
			13 => Ok(EntityStatusEnum::Villager13),
			14 => Ok(EntityStatusEnum::Villager14),
			15 => Ok(EntityStatusEnum::Witch15),
			16 => Ok(EntityStatusEnum::ZombieVillager16),
			17 => Ok(EntityStatusEnum::FireworkRocket17),
			18 => Ok(EntityStatusEnum::AllayAnimal18),
			19 => Ok(EntityStatusEnum::Squid19),
			20 => Ok(EntityStatusEnum::Mob20),
			21 => Ok(EntityStatusEnum::Guardian21),
			22 => Ok(EntityStatusEnum::Player22),
			23 => Ok(EntityStatusEnum::Player23),
			24 => Ok(EntityStatusEnum::Player24),
			25 => Ok(EntityStatusEnum::Player25),
			26 => Ok(EntityStatusEnum::Player26),
			27 => Ok(EntityStatusEnum::Player27),
			28 => Ok(EntityStatusEnum::Player28),
			29 => Ok(EntityStatusEnum::LivingEntity29),
			30 => Ok(EntityStatusEnum::LivingEntity30),
			31 => Ok(EntityStatusEnum::FishingHook31),
			32 => Ok(EntityStatusEnum::ArmorStand32),
			33 => Ok(EntityStatusEnum::Unused33),
			34 => Ok(EntityStatusEnum::IronGolem34),
			35 => Ok(EntityStatusEnum::LivingEntity35),
			36 => Ok(EntityStatusEnum::Unused36),
			37 => Ok(EntityStatusEnum::Unused37),
			38 => Ok(EntityStatusEnum::Dolphin38),
			39 => Ok(EntityStatusEnum::Ravager39),
			40 => Ok(EntityStatusEnum::Ocelot40),
			41 => Ok(EntityStatusEnum::Ocelot41),
			42 => Ok(EntityStatusEnum::Villager42),
			43 => Ok(EntityStatusEnum::Player43),
			44 => Ok(EntityStatusEnum::Unused44),
			45 => Ok(EntityStatusEnum::Fox45),
			46 => Ok(EntityStatusEnum::LivingEntity46),
			47 => Ok(EntityStatusEnum::LivingEntity47),
			48 => Ok(EntityStatusEnum::LivingEntity48),
			49 => Ok(EntityStatusEnum::LivingEntity49),
			50 => Ok(EntityStatusEnum::LivingEntity50),
			51 => Ok(EntityStatusEnum::LivingEntity51),
			52 => Ok(EntityStatusEnum::LivingEntity52),
			53 => Ok(EntityStatusEnum::Entity53),
			54 => Ok(EntityStatusEnum::LivingEntity54),
			55 => Ok(EntityStatusEnum::LivingEntity55),
			56 => Ok(EntityStatusEnum::Wolf56),
			57 => Ok(EntityStatusEnum::Unused57),
			58 => Ok(EntityStatusEnum::Goat58),
			59 => Ok(EntityStatusEnum::Goat59),
			60 => Ok(EntityStatusEnum::LivingEntity60),
			61 => Ok(EntityStatusEnum::Warden61),
			62 => Ok(EntityStatusEnum::Warden62),
			63 => Ok(EntityStatusEnum::Sniffer63),
			_ => Err(SerializingErr::InvalidEnumValue(i)),
		}
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct MinecartMoveStep {
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub velocity_x: f64,
	pub velocity_y: f64,
	pub velocity_z: f64,
	pub yaw: Angle,
	pub pitch: Angle,
	pub weight: f32
}

/// # Position Step (Packet Part)
/// One absolute position in a stepped position path.
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Data_types#Position_Step>
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct PositionStep {
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub tick_offset: VarInt,
}

/// # Position Path (Packet Part)
/// The position data of the Teleport Entity packet. The VarInt "Position Path Type" selects the variant.
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Teleport_Entity>
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum PositionPath {
	#[doc = "One absolute end position"]
	Linear(TripleDouble) = 0,
	#[doc = "A sequence of absolute position steps"]
	Stepped(PrefixedArray<PositionStep>) = 1,
}

/// # Entity Delta Step (Packet Part)
/// One step of a stepped entity movement. Deltas are fixed-point with 12 fractional bits.
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Entity_Position>
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct EntityDeltaStep {
	pub tick_offset: VarInt,
	pub delta_x: i16,
	pub delta_y: i16,
	pub delta_z: i16,
}

/// The movement part of [EntityPositionDelta].
#[derive(Debug, Clone, PartialEq)]
pub enum EntityDeltaMovement {
	/// One X/Y/Z delta. Deltas are fixed-point with 12 fractional bits (currentX * 4096 - prevX * 4096).
	Linear { delta_x: i16, delta_y: i16, delta_z: i16 },
	/// One or more steps. Must not be empty, because an empty list has the same wire form as Linear.
	Stepped(Vec<EntityDeltaStep>),
}

/// # Entity Position Delta (Packet Part)
/// The position data of Update Entity Position and Update Entity Position and Rotation.
///
/// On the wire, a VarInt "Properties" comes first. Bit 0 is On Ground, and bits 1 and above are the
/// step count. If the step count is 0, one X/Y/Z delta follows. If not, that many steps follow.
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Entity_Position>
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPositionDelta {
	pub on_ground: bool,
	pub movement: EntityDeltaMovement,
}

impl McSerialize for EntityPositionDelta {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let step_count = match &self.movement {
			EntityDeltaMovement::Linear { .. } => 0,
			EntityDeltaMovement::Stepped(steps) if steps.is_empty() => {
				return Err(SerializingErr::InconsistentField("Stepped entity movement must have at least one step".to_string()));
			}
			EntityDeltaMovement::Stepped(steps) => steps.len(),
		};

		let step_count = u32::try_from(step_count)
			.ok()
			.filter(|count| *count <= (u32::MAX >> 1))
			.ok_or_else(|| SerializingErr::OutOfBounds(format!("Too many entity movement steps: {}", step_count)))?;
		VarInt(((step_count << 1) | self.on_ground as u32) as i32).mc_serialize(serializer)?;

		match &self.movement {
			EntityDeltaMovement::Linear { delta_x, delta_y, delta_z } => {
				delta_x.mc_serialize(serializer)?;
				delta_y.mc_serialize(serializer)?;
				delta_z.mc_serialize(serializer)?;
			}
			EntityDeltaMovement::Stepped(steps) => {
				for step in steps {
					step.mc_serialize(serializer)?;
				}
			}
		}
		Ok(())
	}
}

impl McDeserialize for EntityPositionDelta {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let properties = VarInt::mc_deserialize(deserializer)?.0 as u32;
		let on_ground = properties & 1 == 1;
		let step_count = properties >> 1;

		let movement = if step_count == 0 {
			EntityDeltaMovement::Linear {
				delta_x: i16::mc_deserialize(deserializer)?,
				delta_y: i16::mc_deserialize(deserializer)?,
				delta_z: i16::mc_deserialize(deserializer)?,
			}
		} else {
			let mut steps = Vec::new();
			for _ in 0..step_count {
				steps.push(EntityDeltaStep::mc_deserialize(deserializer)?);
			}
			EntityDeltaMovement::Stepped(steps)
		};

		Ok(EntityPositionDelta { on_ground, movement })
	}
}

impl McDefault for EntityPositionDelta {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		Self {
			on_ground: false,
			movement: EntityDeltaMovement::Linear { delta_x: 0, delta_y: 0, delta_z: 0 },
		}
	}
}

/// # Swing Animation Type (Packet Part)
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Swing_Animation>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum SwingAnimationType {
	None = 0,
	Whack = 1,
	Stab = 2,
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

	/// Wiki (Update Entity Position): Properties bit 0 is On Ground and bits 1+ are the step count.
	/// With step count 0, three Shorts follow. A wrong bit layout makes the client move entities wrongly.
	#[test]
	fn entity_delta_linear_layout() {
		let delta = EntityPositionDelta { on_ground: true, movement: EntityDeltaMovement::Linear { delta_x: 1, delta_y: -1, delta_z: 256 } };
		assert_eq!(ser(&delta), vec![0x01, 0x00, 0x01, 0xFF, 0xFF, 0x01, 0x00]);
		round_trip(delta);
	}

	/// With step count N > 0, Properties is (N << 1) | on_ground, then N records of
	/// (VarInt tick offset, Short, Short, Short).
	#[test]
	fn entity_delta_stepped_layout() {
		let delta = EntityPositionDelta {
			on_ground: false,
			movement: EntityDeltaMovement::Stepped(vec![
				EntityDeltaStep { tick_offset: VarInt(0), delta_x: 1, delta_y: 2, delta_z: 3 },
				EntityDeltaStep { tick_offset: VarInt(2), delta_x: 4, delta_y: 5, delta_z: 6 },
			]),
		};
		assert_eq!(ser(&delta), vec![0x04, 0, 0, 1, 0, 2, 0, 3, 2, 0, 4, 0, 5, 0, 6]);
		round_trip(delta);
	}

	/// An empty step list has the same wire form as Linear, so it can not round-trip. It must be an error.
	#[test]
	fn entity_delta_empty_steps_is_error() {
		let delta = EntityPositionDelta { on_ground: false, movement: EntityDeltaMovement::Stepped(vec![]) };
		assert!(delta.mc_serialize(&mut McSerializer::new()).is_err());
	}

	/// Wiki (Teleport Entity): Position Path Type 0 is followed by X/Y/Z, type 1 by a Prefixed Array of Position Step.
	#[test]
	fn position_path_layout() {
		let linear = PositionPath::Linear(TripleDouble { x: 1.0, y: 2.0, z: 3.0 });
		let mut expected = vec![0];
		for v in [1.0f64, 2.0, 3.0] {
			expected.extend_from_slice(&v.to_be_bytes());
		}
		assert_eq!(ser(&linear), expected);
		round_trip(linear);

		let stepped = PositionPath::Stepped(PrefixedArray::new(vec![PositionStep { x: 1.0, y: 2.0, z: 3.0, tick_offset: VarInt(5) }]));
		let mut expected = vec![1, 1];
		for v in [1.0f64, 2.0, 3.0] {
			expected.extend_from_slice(&v.to_be_bytes());
		}
		expected.push(5);
		assert_eq!(ser(&stepped), expected);
		round_trip(stepped);
	}

	#[test]
	fn entity_status_is_one_signed_byte() {
		// Entity Event sends the status as a Byte, not a VarInt.
		for (status, byte) in [(EntityStatusEnum::Arrow0, 0u8), (EntityStatusEnum::Player9, 9), (EntityStatusEnum::Warden62, 62), (EntityStatusEnum::Sniffer63, 63)] {
			let mut s = McSerializer::new();
			status.mc_serialize(&mut s).unwrap();
			assert_eq!(s.as_bytes(), &[byte]);
			assert_eq!(EntityStatusEnum::mc_deserialize(&mut McDeserializer::new(&[byte])).unwrap(), status);
		}
	}

	/// The read table is written by hand, separate from the enum discriminants. One wrong arm would
	/// decode a status as a different event. Each variant name ends with its code, so for every
	/// code: the name suffix, the discriminant, and the re-serialized byte must all equal the code.
	#[test]
	fn entity_status_table_matches_discriminants_and_names() {
		for code in 0i8..=63 {
			let status = EntityStatusEnum::mc_deserialize(&mut McDeserializer::new(&[code as u8])).unwrap_or_else(|e| panic!("code {code}: {e:?}"));
			assert_eq!(status as i8, code, "{status:?}");

			let name = format!("{status:?}");
			let suffix: String = name.chars().rev().take_while(|c| c.is_ascii_digit()).collect::<Vec<_>>().into_iter().rev().collect();
			assert_eq!(suffix.parse::<i8>().ok(), Some(code), "variant {name} is decoded from code {code}");

			let mut s = McSerializer::new();
			status.mc_serialize(&mut s).unwrap();
			assert_eq!(s.as_bytes(), &[code as u8]);
		}
	}

	#[test]
	fn unknown_entity_status_is_an_error() {
		// 64 is the first code past the table, and a negative byte is never valid.
		for byte in [64u8, 0xFF] {
			assert!(EntityStatusEnum::mc_deserialize(&mut McDeserializer::new(&[byte])).is_err(), "byte {byte}");
		}
		assert!(EntityStatusEnum::mc_deserialize(&mut McDeserializer::new(&[100])).is_err());
	}
}
