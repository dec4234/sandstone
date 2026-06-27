use crate::protocol::game::effects::particle::Particle;
use crate::protocol::game::player::inventory::slotdata::SlotData;
use crate::protocol::packets::packet_parts::item::IdOrPaintingVariant;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::game_types::Position;
use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};
use uuid::Uuid;

#[derive(VarIntEnum, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum EntityMetadataValue {
	Byte(u8) = 0,
	VarInt(VarInt) = 1,
	VarLong(VarLong) = 2,
	Float(f32) = 3,
	String(String) = 4,
	TextComponent(TextComponent) = 5,
	OptionalTextComponent(PrefixedOptional<TextComponent>) = 6,
	Slot(SlotData) = 7,
	Boolean(bool) = 8,
	Rotations(f32, f32, f32) = 9,
	Position(Position) = 10,
	OptionalPosition(PrefixedOptional<Position>) = 11,
	Direction(VarInt) = 12,
	OptionalLivingEntityRef(PrefixedOptional<Uuid>) = 13,
	BlockState(VarInt) = 14,
	OptionalBlockState(VarInt) = 15,
	Particle(Particle) = 16,
	Particles(PrefixedArray<Particle>) = 17,
	VillagerData(VarInt, VarInt, VarInt) = 18,
	OptionalVarInt(VarInt) = 19,
	Pose(VarInt) = 20,
	CatVariant(VarInt) = 21,
	CowVariant(VarInt) = 22,
	WolfVariant(VarInt) = 23,
	WolfSoundVariant(VarInt) = 24,
	FrogVariant(VarInt) = 25,
	PigVariant(VarInt) = 26,
	ChickenVariant(VarInt) = 27,
	ZombieNautilusVariant(VarInt) = 28,
	OptionalGlobalPosition(PrefixedOptional<GlobalPosition>) = 29,
	PaintingVariant(IdOrPaintingVariant) = 30,
	SnifferState(VarInt) = 31,
	ArmadilloState(VarInt) = 32,
	CopperGolemState(VarInt) = 33,
	WeatheringCopperState(VarInt) = 34,
	Vector3(f32, f32, f32) = 35,
	Quaternion(f32, f32, f32, f32) = 36,
	ResolvableProfile(ResolvableProfile) = 37,
	HumanoidArm(VarInt) = 38,
}

impl McDefault for EntityMetadataValue {
	fn mc_default() -> Self {
		EntityMetadataValue::Byte(0)
	}
}

/// The dimension/position pair carried by [EntityMetadataValue::OptionalGlobalPosition].
#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct GlobalPosition {
	pub dimension: String,
	pub position: Position,
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
			entries.push(EntityMetadataEntry {
				index,
				value,
			});
		}
		Ok(EntityMetadata {
			entries,
		})
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ProfileProperty {
	pub name: String,
	pub value: String,
	pub signature: PrefixedOptional<String>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ResolvableProfile {
	pub name: PrefixedOptional<String>,
	pub uuid: PrefixedOptional<Uuid>,
	pub properties: PrefixedArray<ProfileProperty>,
}
