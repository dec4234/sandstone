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
	fn metadata_ends_with_0xff() {
		// Wiki (Entity Metadata): the list has no length prefix. Index 0xFF ends it, so a missing
		// terminator makes the client read into the next packet.
		assert_eq!(ser(&EntityMetadata { entries: vec![] }), vec![0xFF]);
		let m = EntityMetadata { entries: vec![EntityMetadataEntry { index: 0, value: EntityMetadataValue::Byte(0x20) }] };
		assert_eq!(ser(&m), vec![0, 0, 0x20, 0xFF]);
	}

	#[test]
	fn metadata_stops_reading_at_terminator() {
		let mut d = McDeserializer::new(&[3, 8, 1, 0xFF, 0xEE]);
		let m = EntityMetadata::mc_deserialize(&mut d).unwrap();
		assert_eq!(m.entries, vec![EntityMetadataEntry { index: 3, value: EntityMetadataValue::Boolean(true) }]);
		assert_eq!(d.collect_remaining(), &[0xEE]);
	}

	#[test]
	fn metadata_without_terminator_is_an_error() {
		assert!(EntityMetadata::mc_deserialize(&mut McDeserializer::new(&[0, 0, 1])).is_err());
	}

	#[test]
	fn value_type_ids_match_wiki() {
		assert_eq!(ser(&EntityMetadataValue::VarInt(VarInt(1))), vec![1, 1]);
		assert_eq!(ser(&EntityMetadataValue::Boolean(false)), vec![8, 0]);
		assert_eq!(ser(&EntityMetadataValue::Pose(VarInt(5))), vec![20, 5]);
		assert_eq!(ser(&EntityMetadataValue::HumanoidArm(VarInt(1))), vec![38, 1]);
	}

	#[test]
	fn many_value_types_round_trip() {
		let empty_slot = SlotData { item_count: VarInt(0), item_id: None, components_to_add: vec![], components_to_remove: vec![] };
		let m = EntityMetadata {
			entries: vec![
				EntityMetadataEntry { index: 0, value: EntityMetadataValue::Byte(0x02) },
				EntityMetadataEntry { index: 1, value: EntityMetadataValue::VarLong(VarLong(1 << 40)) },
				EntityMetadataEntry { index: 2, value: EntityMetadataValue::Float(0.5) },
				EntityMetadataEntry { index: 3, value: EntityMetadataValue::String("s".into()) },
				EntityMetadataEntry { index: 4, value: EntityMetadataValue::TextComponent(TextComponent::new("name")) },
				EntityMetadataEntry { index: 5, value: EntityMetadataValue::OptionalTextComponent(PrefixedOptional::new(None)) },
				EntityMetadataEntry { index: 6, value: EntityMetadataValue::Slot(empty_slot) },
				EntityMetadataEntry { index: 7, value: EntityMetadataValue::Rotations(1.0, 2.0, 3.0) },
				EntityMetadataEntry { index: 8, value: EntityMetadataValue::OptionalPosition(PrefixedOptional::new(Some(Position::new(1, -2, 3)))) },
				EntityMetadataEntry { index: 9, value: EntityMetadataValue::OptionalLivingEntityRef(PrefixedOptional::new(Some(Uuid::from_u128(7)))) },
				EntityMetadataEntry { index: 10, value: EntityMetadataValue::Particle(Particle::Flame) },
				EntityMetadataEntry { index: 11, value: EntityMetadataValue::Particles(PrefixedArray::new(vec![Particle::Dust(0xFF0000, 1.0)])) },
				EntityMetadataEntry { index: 12, value: EntityMetadataValue::VillagerData(VarInt(1), VarInt(2), VarInt(3)) },
				EntityMetadataEntry {
					index: 13,
					value: EntityMetadataValue::OptionalGlobalPosition(PrefixedOptional::new(Some(GlobalPosition { dimension: "minecraft:overworld".into(), position: Position::new(0, 64, 0) }))),
				},
				EntityMetadataEntry { index: 14, value: EntityMetadataValue::Quaternion(0.0, 0.0, 0.0, 1.0) },
				EntityMetadataEntry {
					index: 15,
					value: EntityMetadataValue::ResolvableProfile(ResolvableProfile {
						name: PrefixedOptional::new(Some("Steve".into())),
						uuid: PrefixedOptional::new(None),
						properties: PrefixedArray::new(vec![ProfileProperty { name: "textures".into(), value: "abc".into(), signature: PrefixedOptional::new(None) }]),
					}),
				},
			],
		};
		round_trip(m);
	}
}
