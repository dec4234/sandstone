use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;

/// The status code sent in the Entity Event packet, represented as a byte.
///
/// Each code links to the entity (or entities) it applies to. The meaning of a
/// given code differs per entity, so variants are suffixed with their numeric
/// code to keep them unambiguous.
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Entity_statuses
#[derive(Debug, Clone, Hash, PartialEq)]
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
	fn mc_default() -> Self {
		Self::Arrow0
	}
}

impl McSerialize for EntityStatusEnum {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		(self.clone() as i8).mc_serialize(serializer)
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