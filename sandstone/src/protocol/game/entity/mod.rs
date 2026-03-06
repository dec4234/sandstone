pub mod particle;

use crate::protocol::game::entity::particle::Particle;
use crate::protocol::game::info::inventory::component_types::{IdOrPaintingVariant, ResolvableProfile};
use crate::protocol::game::info::inventory::slotdata::SlotData;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::serialization::{
	McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult,
};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::game_types::Position;
use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};
use sandstone_derive::McDefault;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum EntityMetadataValue {
	Byte(u8),                                                    // 0
	VarInt(VarInt),                                              // 1
	VarLong(VarLong),                                            // 2
	Float(f32),                                                  // 3
	String(String),                                              // 4
	TextComponent(TextComponent),                                // 5
	OptionalTextComponent(Option<TextComponent>),                // 6
	Slot(SlotData),                                              // 7
	Boolean(bool),                                               // 8
	Rotations(f32, f32, f32),                                    // 9
	Position(Position),                                          // 10
	OptionalPosition(Option<Position>),                          // 11
	Direction(VarInt),                                            // 12
	OptionalLivingEntityRef(Option<Uuid>),                       // 13
	BlockState(VarInt),                                          // 14
	OptionalBlockState(VarInt),                                  // 15
	Particle(Particle),                                          // 16
	Particles(PrefixedArray<Particle>),                          // 17
	VillagerData(VarInt, VarInt, VarInt),                        // 18
	OptionalVarInt(VarInt),                                      // 19
	Pose(VarInt),                                                // 20
	CatVariant(VarInt),                                          // 21
	CowVariant(VarInt),                                          // 22
	WolfVariant(VarInt),                                         // 23
	WolfSoundVariant(VarInt),                                    // 24
	FrogVariant(VarInt),                                         // 25
	PigVariant(VarInt),                                          // 26
	ChickenVariant(VarInt),                                      // 27
	ZombieNautilusVariant(VarInt),                               // 28
	OptionalGlobalPosition(Option<(String, Position)>),          // 29
	PaintingVariant(IdOrPaintingVariant),                        // 30
	SnifferState(VarInt),                                        // 31
	ArmadilloState(VarInt),                                      // 32
	CopperGolemState(VarInt),                                    // 33
	WeatheringCopperState(VarInt),                               // 34
	Vector3(f32, f32, f32),                                      // 35
	Quaternion(f32, f32, f32, f32),                               // 36
	ResolvableProfile(ResolvableProfile),                        // 37
	HumanoidArm(VarInt),                                         // 38
}

impl McDefault for EntityMetadataValue {
	fn mc_default() -> Self {
		EntityMetadataValue::Byte(0)
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityMetadataEntry {
	pub index: u8,
	pub value: EntityMetadataValue,
}

impl McDefault for EntityMetadataEntry {
	fn mc_default() -> Self {
		Self {
			index: 0,
			value: EntityMetadataValue::mc_default(),
		}
	}
}

#[derive(McDefault, Debug, Clone, PartialEq)]
pub struct EntityMetadata {
	pub entries: Vec<EntityMetadataEntry>,
}

impl McSerialize for EntityMetadataValue {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			EntityMetadataValue::Byte(v) => {
				VarInt(0).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::VarInt(v) => {
				VarInt(1).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::VarLong(v) => {
				VarInt(2).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::Float(v) => {
				VarInt(3).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::String(v) => {
				VarInt(4).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::TextComponent(v) => {
				VarInt(5).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::OptionalTextComponent(v) => {
				VarInt(6).mc_serialize(serializer)?;
				match v {
					Some(tc) => {
						true.mc_serialize(serializer)?;
						tc.mc_serialize(serializer)?;
					}
					None => {
						false.mc_serialize(serializer)?;
					}
				}
			}
			EntityMetadataValue::Slot(v) => {
				VarInt(7).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::Boolean(v) => {
				VarInt(8).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::Rotations(x, y, z) => {
				VarInt(9).mc_serialize(serializer)?;
				x.mc_serialize(serializer)?;
				y.mc_serialize(serializer)?;
				z.mc_serialize(serializer)?;
			}
			EntityMetadataValue::Position(v) => {
				VarInt(10).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::OptionalPosition(v) => {
				VarInt(11).mc_serialize(serializer)?;
				match v {
					Some(pos) => {
						true.mc_serialize(serializer)?;
						pos.mc_serialize(serializer)?;
					}
					None => {
						false.mc_serialize(serializer)?;
					}
				}
			}
			EntityMetadataValue::Direction(v) => {
				VarInt(12).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::OptionalLivingEntityRef(v) => {
				VarInt(13).mc_serialize(serializer)?;
				match v {
					Some(uuid) => {
						true.mc_serialize(serializer)?;
						uuid.mc_serialize(serializer)?;
					}
					None => {
						false.mc_serialize(serializer)?;
					}
				}
			}
			EntityMetadataValue::BlockState(v) => {
				VarInt(14).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::OptionalBlockState(v) => {
				VarInt(15).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::Particle(v) => {
				VarInt(16).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::Particles(v) => {
				VarInt(17).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::VillagerData(typ, profession, level) => {
				VarInt(18).mc_serialize(serializer)?;
				typ.mc_serialize(serializer)?;
				profession.mc_serialize(serializer)?;
				level.mc_serialize(serializer)?;
			}
			EntityMetadataValue::OptionalVarInt(v) => {
				VarInt(19).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::Pose(v) => {
				VarInt(20).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::CatVariant(v) => {
				VarInt(21).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::CowVariant(v) => {
				VarInt(22).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::WolfVariant(v) => {
				VarInt(23).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::WolfSoundVariant(v) => {
				VarInt(24).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::FrogVariant(v) => {
				VarInt(25).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::PigVariant(v) => {
				VarInt(26).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::ChickenVariant(v) => {
				VarInt(27).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::ZombieNautilusVariant(v) => {
				VarInt(28).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::OptionalGlobalPosition(v) => {
				VarInt(29).mc_serialize(serializer)?;
				match v {
					Some((dimension, pos)) => {
						true.mc_serialize(serializer)?;
						dimension.mc_serialize(serializer)?;
						pos.mc_serialize(serializer)?;
					}
					None => {
						false.mc_serialize(serializer)?;
					}
				}
			}
			EntityMetadataValue::PaintingVariant(v) => {
				VarInt(30).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::SnifferState(v) => {
				VarInt(31).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::ArmadilloState(v) => {
				VarInt(32).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::CopperGolemState(v) => {
				VarInt(33).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::WeatheringCopperState(v) => {
				VarInt(34).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::Vector3(x, y, z) => {
				VarInt(35).mc_serialize(serializer)?;
				x.mc_serialize(serializer)?;
				y.mc_serialize(serializer)?;
				z.mc_serialize(serializer)?;
			}
			EntityMetadataValue::Quaternion(x, y, z, w) => {
				VarInt(36).mc_serialize(serializer)?;
				x.mc_serialize(serializer)?;
				y.mc_serialize(serializer)?;
				z.mc_serialize(serializer)?;
				w.mc_serialize(serializer)?;
			}
			EntityMetadataValue::ResolvableProfile(v) => {
				VarInt(37).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
			EntityMetadataValue::HumanoidArm(v) => {
				VarInt(38).mc_serialize(serializer)?;
				v.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for EntityMetadataValue {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		match typ {
			0 => Ok(EntityMetadataValue::Byte(u8::mc_deserialize(deserializer)?)),
			1 => Ok(EntityMetadataValue::VarInt(VarInt::mc_deserialize(deserializer)?)),
			2 => Ok(EntityMetadataValue::VarLong(VarLong::mc_deserialize(deserializer)?)),
			3 => Ok(EntityMetadataValue::Float(f32::mc_deserialize(deserializer)?)),
			4 => Ok(EntityMetadataValue::String(String::mc_deserialize(deserializer)?)),
			5 => Ok(EntityMetadataValue::TextComponent(TextComponent::mc_deserialize(deserializer)?)),
			6 => {
				let present = bool::mc_deserialize(deserializer)?;
				let tc = if present {
					Some(TextComponent::mc_deserialize(deserializer)?)
				} else {
					None
				};
				Ok(EntityMetadataValue::OptionalTextComponent(tc))
			}
			7 => Ok(EntityMetadataValue::Slot(SlotData::mc_deserialize(deserializer)?)),
			8 => Ok(EntityMetadataValue::Boolean(bool::mc_deserialize(deserializer)?)),
			9 => {
				let x = f32::mc_deserialize(deserializer)?;
				let y = f32::mc_deserialize(deserializer)?;
				let z = f32::mc_deserialize(deserializer)?;
				Ok(EntityMetadataValue::Rotations(x, y, z))
			}
			10 => Ok(EntityMetadataValue::Position(Position::mc_deserialize(deserializer)?)),
			11 => {
				let present = bool::mc_deserialize(deserializer)?;
				let pos = if present {
					Some(Position::mc_deserialize(deserializer)?)
				} else {
					None
				};
				Ok(EntityMetadataValue::OptionalPosition(pos))
			}
			12 => Ok(EntityMetadataValue::Direction(VarInt::mc_deserialize(deserializer)?)),
			13 => {
				let present = bool::mc_deserialize(deserializer)?;
				let uuid = if present {
					Some(Uuid::mc_deserialize(deserializer)?)
				} else {
					None
				};
				Ok(EntityMetadataValue::OptionalLivingEntityRef(uuid))
			}
			14 => Ok(EntityMetadataValue::BlockState(VarInt::mc_deserialize(deserializer)?)),
			15 => Ok(EntityMetadataValue::OptionalBlockState(VarInt::mc_deserialize(deserializer)?)),
			16 => Ok(EntityMetadataValue::Particle(Particle::mc_deserialize(deserializer)?)),
			17 => Ok(EntityMetadataValue::Particles(PrefixedArray::mc_deserialize(deserializer)?)),
			18 => {
				let typ = VarInt::mc_deserialize(deserializer)?;
				let profession = VarInt::mc_deserialize(deserializer)?;
				let level = VarInt::mc_deserialize(deserializer)?;
				Ok(EntityMetadataValue::VillagerData(typ, profession, level))
			}
			19 => Ok(EntityMetadataValue::OptionalVarInt(VarInt::mc_deserialize(deserializer)?)),
			20 => Ok(EntityMetadataValue::Pose(VarInt::mc_deserialize(deserializer)?)),
			21 => Ok(EntityMetadataValue::CatVariant(VarInt::mc_deserialize(deserializer)?)),
			22 => Ok(EntityMetadataValue::CowVariant(VarInt::mc_deserialize(deserializer)?)),
			23 => Ok(EntityMetadataValue::WolfVariant(VarInt::mc_deserialize(deserializer)?)),
			24 => Ok(EntityMetadataValue::WolfSoundVariant(VarInt::mc_deserialize(deserializer)?)),
			25 => Ok(EntityMetadataValue::FrogVariant(VarInt::mc_deserialize(deserializer)?)),
			26 => Ok(EntityMetadataValue::PigVariant(VarInt::mc_deserialize(deserializer)?)),
			27 => Ok(EntityMetadataValue::ChickenVariant(VarInt::mc_deserialize(deserializer)?)),
			28 => Ok(EntityMetadataValue::ZombieNautilusVariant(VarInt::mc_deserialize(deserializer)?)),
			29 => {
				let present = bool::mc_deserialize(deserializer)?;
				if present {
					let dimension = String::mc_deserialize(deserializer)?;
					let pos = Position::mc_deserialize(deserializer)?;
					Ok(EntityMetadataValue::OptionalGlobalPosition(Some((dimension, pos))))
				} else {
					Ok(EntityMetadataValue::OptionalGlobalPosition(None))
				}
			}
			30 => Ok(EntityMetadataValue::PaintingVariant(IdOrPaintingVariant::mc_deserialize(deserializer)?)),
			31 => Ok(EntityMetadataValue::SnifferState(VarInt::mc_deserialize(deserializer)?)),
			32 => Ok(EntityMetadataValue::ArmadilloState(VarInt::mc_deserialize(deserializer)?)),
			33 => Ok(EntityMetadataValue::CopperGolemState(VarInt::mc_deserialize(deserializer)?)),
			34 => Ok(EntityMetadataValue::WeatheringCopperState(VarInt::mc_deserialize(deserializer)?)),
			35 => {
				let x = f32::mc_deserialize(deserializer)?;
				let y = f32::mc_deserialize(deserializer)?;
				let z = f32::mc_deserialize(deserializer)?;
				Ok(EntityMetadataValue::Vector3(x, y, z))
			}
			36 => {
				let x = f32::mc_deserialize(deserializer)?;
				let y = f32::mc_deserialize(deserializer)?;
				let z = f32::mc_deserialize(deserializer)?;
				let w = f32::mc_deserialize(deserializer)?;
				Ok(EntityMetadataValue::Quaternion(x, y, z, w))
			}
			37 => Ok(EntityMetadataValue::ResolvableProfile(ResolvableProfile::mc_deserialize(deserializer)?)),
			38 => Ok(EntityMetadataValue::HumanoidArm(VarInt::mc_deserialize(deserializer)?)),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid EntityMetadataValue type: {}", typ))),
		}
	}
}

impl McSerialize for EntityMetadata {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		for entry in &self.entries {
			entry.index.mc_serialize(serializer)?;
			entry.value.mc_serialize(serializer)?;
		}
		0xFFu8.mc_serialize(serializer)?;
		Ok(())
	}
}

impl McDeserialize for EntityMetadata {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let mut entries = Vec::new();
		loop {
			let index = u8::mc_deserialize(deserializer)?;
			if index == 0xFF {
				break;
			}
			let value = EntityMetadataValue::mc_deserialize(deserializer)?;
			entries.push(EntityMetadataEntry { index, value });
		}
		Ok(EntityMetadata { entries })
	}
}