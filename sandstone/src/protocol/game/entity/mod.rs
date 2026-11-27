use crate::protocol::game::effects::particle::Particle;
use crate::protocol::game::player::inventory::slotdata::SlotData;
use crate::protocol::packets::packet_parts::item::IdOrPaintingVariant;
use crate::protocol::packets::packet_parts::item_modifiers::DyeColor;
use crate::protocol::packets::packet_parts::player::MainHand;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::game_types::Position;
use crate::protocol_types::datatypes::internal_types::Identifier;
use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};
use uuid::Uuid;

/// The type IDs are the registration order in `EntityDataSerializers` (verified against the 26.3
/// server jar).
///
/// <https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata#Entity_Metadata_Format>
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
	/// Rotation on x, y, z.
	Rotations(f32, f32, f32) = 9,
	Position(Position) = 10,
	OptionalPosition(PrefixedOptional<Position>) = 11,
	Direction(Direction) = 12,
	OptionalLivingEntityRef(PrefixedOptional<Uuid>) = 13,
	/// An ID in the block state registry.
	BlockState(VarInt) = 14,
	/// 0 for absent (air cannot be sent); otherwise, an ID in the block state registry.
	OptionalBlockState(VarInt) = 15,
	Particle(Particle) = 16,
	Particles(PrefixedArray<Particle>) = 17,
	VillagerData(VillagerData) = 18,
	/// 0 for absent; otherwise 1 + the value. Used for entity IDs.
	OptionalVarInt(VarInt) = 19,
	Pose(Pose) = 20,
	/// An ID in the `minecraft:cat_variant` registry.
	CatVariant(VarInt) = 21,
	/// An ID in the `minecraft:cat_sound_variant` registry.
	CatSoundVariant(VarInt) = 22,
	/// An ID in the `minecraft:cow_variant` registry.
	CowVariant(VarInt) = 23,
	/// An ID in the `minecraft:cow_sound_variant` registry.
	CowSoundVariant(VarInt) = 24,
	/// An ID in the `minecraft:wolf_variant` registry.
	WolfVariant(VarInt) = 25,
	/// An ID in the `minecraft:wolf_sound_variant` registry.
	WolfSoundVariant(VarInt) = 26,
	/// An ID in the `minecraft:frog_variant` registry.
	FrogVariant(VarInt) = 27,
	/// An ID in the `minecraft:pig_variant` registry.
	PigVariant(VarInt) = 28,
	/// An ID in the `minecraft:pig_sound_variant` registry.
	PigSoundVariant(VarInt) = 29,
	/// An ID in the `minecraft:chicken_variant` registry.
	ChickenVariant(VarInt) = 30,
	/// An ID in the `minecraft:chicken_sound_variant` registry.
	ChickenSoundVariant(VarInt) = 31,
	/// An ID in the `minecraft:zombie_nautilus_variant` registry.
	ZombieNautilusVariant(VarInt) = 32,
	OptionalGlobalPosition(PrefixedOptional<GlobalPosition>) = 33,
	PaintingVariant(IdOrPaintingVariant) = 34,
	SnifferState(SnifferState) = 35,
	ArmadilloState(ArmadilloState) = 36,
	CopperGolemState(CopperGolemState) = 37,
	WeatheringCopperState(WeatheringCopperState) = 38,
	/// x, y, z
	Vector3(f32, f32, f32) = 39,
	/// x, y, z, w
	Quaternion(f32, f32, f32, f32) = 40,
	ResolvableProfile(ResolvableProfile) = 41,
	/// The wire values are the same as the client setting, so this uses [MainHand].
	HumanoidArm(MainHand) = 42,
	DyeColor(DyeColor) = 43,
}

impl McDefault for EntityMetadataValue {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		EntityMetadataValue::Byte(0)
	}
}

/// The dimension/position pair carried by [EntityMetadataValue::OptionalGlobalPosition].
#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct GlobalPosition {
	pub dimension: Identifier,
	pub position: Position,
}

/// The data carried by [EntityMetadataValue::VillagerData].
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct VillagerData {
	/// An ID in the `minecraft:villager_type` registry.
	pub villager_type: VarInt,
	/// An ID in the `minecraft:villager_profession` registry.
	pub profession: VarInt,
	pub level: VarInt,
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata#Entity_Metadata_Format>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum Direction {
	Down = 0,
	Up = 1,
	North = 2,
	South = 3,
	West = 4,
	East = 5,
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata#Entity_Metadata_Format>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum Pose {
	Standing = 0,
	FallFlying = 1,
	Sleeping = 2,
	Swimming = 3,
	SpinAttack = 4,
	Sneaking = 5,
	LongJumping = 6,
	Dying = 7,
	Croaking = 8,
	UsingTongue = 9,
	Sitting = 10,
	Roaring = 11,
	Sniffing = 12,
	Emerging = 13,
	Digging = 14,
	Sliding = 15,
	Shooting = 16,
	Inhaling = 17,
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata#Entity_Metadata_Format>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum SnifferState {
	Idling = 0,
	FeelingHappy = 1,
	Scenting = 2,
	Sniffing = 3,
	Searching = 4,
	Digging = 5,
	Rising = 6,
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata#Entity_Metadata_Format>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum ArmadilloState {
	Idle = 0,
	Rolling = 1,
	Scared = 2,
	Unrolling = 3,
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata#Entity_Metadata_Format>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum CopperGolemState {
	Idle = 0,
	GettingItem = 1,
	GettingNoItem = 2,
	DroppingItem = 3,
	DroppingNoItem = 4,
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata#Entity_Metadata_Format>
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
pub enum WeatheringCopperState {
	Unaffected = 0,
	Exposed = 1,
	Weathered = 2,
	Oxidized = 3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityMetadataEntry {
	pub index: u8,
	pub value: EntityMetadataValue,
}

impl McDefault for EntityMetadataEntry {
	#[cfg(any(test, feature = "testing"))]
	fn mc_default() -> Self {
		Self {
			index: 0,
			value: EntityMetadataValue::mc_default(),
		}
	}
}

/// <https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata#Entity_Metadata_Format>
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

/// <https://minecraft.wiki/w/Java_Edition_protocol/Data_types#Resolvable_Profile>
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
	fn value_type_ids_match_vanilla() {
		// IDs from the EntityDataSerializers registration order in the 26.3 server jar. A wrong ID
		// makes the client read the value with the wrong type and disconnect.
		assert_eq!(ser(&EntityMetadataValue::VarInt(VarInt(1))), vec![1, 1]);
		assert_eq!(ser(&EntityMetadataValue::Boolean(false)), vec![8, 0]);
		assert_eq!(ser(&EntityMetadataValue::Direction(Direction::East)), vec![12, 5]);
		assert_eq!(ser(&EntityMetadataValue::Pose(Pose::Sneaking)), vec![20, 5]);
		assert_eq!(ser(&EntityMetadataValue::CatSoundVariant(VarInt(0))), vec![22, 0]);
		assert_eq!(ser(&EntityMetadataValue::ChickenSoundVariant(VarInt(0))), vec![31, 0]);
		assert_eq!(ser(&EntityMetadataValue::ZombieNautilusVariant(VarInt(0))), vec![32, 0]);
		assert_eq!(ser(&EntityMetadataValue::SnifferState(SnifferState::Rising)), vec![35, 6]);
		assert_eq!(ser(&EntityMetadataValue::ArmadilloState(ArmadilloState::Unrolling)), vec![36, 3]);
		assert_eq!(ser(&EntityMetadataValue::CopperGolemState(CopperGolemState::DroppingNoItem)), vec![37, 4]);
		assert_eq!(ser(&EntityMetadataValue::WeatheringCopperState(WeatheringCopperState::Oxidized)), vec![38, 3]);
		assert_eq!(ser(&EntityMetadataValue::HumanoidArm(MainHand::Right)), vec![42, 1]);
		assert_eq!(ser(&EntityMetadataValue::DyeColor(DyeColor::Black)), vec![43, 15]);
	}

	#[test]
	fn subtype_values_match_vanilla() {
		// Values are the enum ordinals in the 26.3 server jar. The last value of each enum checks
		// that no value is missing in the middle.
		assert_eq!(ser(&Pose::Standing), vec![0]);
		assert_eq!(ser(&Pose::Inhaling), vec![17]);
		assert_eq!(ser(&Direction::Down), vec![0]);
		assert_eq!(ser(&SnifferState::Idling), vec![0]);
		assert_eq!(ser(&CopperGolemState::Idle), vec![0]);
		assert_eq!(ser(&WeatheringCopperState::Unaffected), vec![0]);
	}

	#[test]
	fn unknown_subtype_value_is_an_error() {
		// A value past the end means the protocol changed. It must fail, not map to a wrong state.
		assert!(Pose::mc_deserialize(&mut McDeserializer::new(&[18])).is_err());
		assert!(Direction::mc_deserialize(&mut McDeserializer::new(&[6])).is_err());
		assert!(EntityMetadataValue::mc_deserialize(&mut McDeserializer::new(&[44, 0])).is_err());
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
				EntityMetadataEntry {
					index: 12,
					value: EntityMetadataValue::VillagerData(VillagerData { villager_type: VarInt(1), profession: VarInt(2), level: VarInt(3) }),
				},
				EntityMetadataEntry {
					index: 13,
					value: EntityMetadataValue::OptionalGlobalPosition(PrefixedOptional::new(Some(GlobalPosition { dimension: "minecraft:overworld".parse().unwrap(), position: Position::new(0, 64, 0) }))),
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
				EntityMetadataEntry { index: 16, value: EntityMetadataValue::Pose(Pose::Swimming) },
				EntityMetadataEntry { index: 17, value: EntityMetadataValue::Direction(Direction::West) },
				EntityMetadataEntry { index: 18, value: EntityMetadataValue::ArmadilloState(ArmadilloState::Scared) },
				EntityMetadataEntry { index: 19, value: EntityMetadataValue::HumanoidArm(MainHand::Left) },
				EntityMetadataEntry { index: 20, value: EntityMetadataValue::DyeColor(DyeColor::Red) },
				EntityMetadataEntry { index: 21, value: EntityMetadataValue::PigSoundVariant(VarInt(2)) },
			],
		};
		round_trip(m);
	}
}
