use crate::bitflag;
use crate::protocol::game::effects::particle::Particle;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::internal_types::IDSet;
use crate::protocol_types::datatypes::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, TypeEnum, VarIntEnum};

bitflag!(CommandBlockFlag: u8 {
	track_output, is_conditional, automatic
});

bitflag!(StructureBlockFlags: u8 {
	ignore_entities, show_air, show_bounding_box, strict_placement
});

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct BlockParticleAlternative {
	pub particle_id: VarInt,
	pub particle_data: Particle,
	pub scaling: f32,
	pub speed: f32,
	pub weight: VarInt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockPredicate {
	pub blocks: PrefixedOptional<IDSet>,
	pub properties: PrefixedOptional<PrefixedArray<BlockProperty>>,
	pub nbt: PrefixedOptional<NbtCompound>,
}

impl McSerialize for BlockPredicate {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.blocks.mc_serialize(serializer)?;
		self.properties.mc_serialize(serializer)?;
		self.nbt.mc_serialize(serializer)?;
		Ok(())
	}
}

impl McDeserialize for BlockPredicate {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		Ok(Self {
			blocks: PrefixedOptional::mc_deserialize(deserializer)?,
			properties: PrefixedOptional::mc_deserialize(deserializer)?,
			nbt: PrefixedOptional::mc_deserialize(deserializer)?,
		})
	}
}

impl McDefault for BlockPredicate {
	fn mc_default() -> Self {
		Self {
			blocks: PrefixedOptional::new(None),
			properties: PrefixedOptional::new(None),
			nbt: PrefixedOptional::new(None),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockProperty {
	pub name: String,
	pub is_exact: bool,
	pub exact_value: Option<String>,
	pub min_value: Option<String>,
	pub max_value: Option<String>,
}

impl McSerialize for BlockProperty {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.name.mc_serialize(serializer)?;
		self.is_exact.mc_serialize(serializer)?;
		if self.is_exact {
			self.exact_value.mc_serialize(serializer)?;
		} else {
			self.min_value.mc_serialize(serializer)?;
			self.max_value.mc_serialize(serializer)?;
		}
		Ok(())
	}
}

impl McDeserialize for BlockProperty {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let name = String::mc_deserialize(deserializer)?;
		let is_exact = bool::mc_deserialize(deserializer)?;
		if is_exact {
			Ok(Self {
				name,
				is_exact,
				exact_value: Some(String::mc_deserialize(deserializer)?),
				min_value: None,
				max_value: None,
			})
		} else {
			Ok(Self {
				name,
				is_exact,
				exact_value: None,
				min_value: Some(String::mc_deserialize(deserializer)?),
				max_value: Some(String::mc_deserialize(deserializer)?),
			})
		}
	}
}

impl McDefault for BlockProperty {
	fn mc_default() -> Self {
		Self {
			name: String::mc_default(),
			is_exact: true,
			exact_value: Some(String::mc_default()),
			min_value: None,
			max_value: None,
		}
	}
}

/// # Command Block Mode (Packet Part)
/// The mode for a command block
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Command_Block
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum CommandBlockMode {
	Chain = 0,
	Repeating = 1,
	Impulse = 2,
}

/// # Structure Block Action (Packet Part)
/// The action being undertaken by the stucture block
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Structure_Block
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum StructureBlockAction {
	UpdateData = 0,
	SaveStructure = 1,
	LoadStructure = 2,
	DetectSize = 3,
}

/// # Structure Block Mode (Packet Part)
/// The mode of the structure block
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Structure_Block
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum StructureBlockMode {
	Save = 0,
	Load = 1,
	Corner = 2,
	Data = 3,
}

/// # Structure Block Mirror State (Packet Part)
/// The mirror state of the structure block
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Structure_Block
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum StructureBlockMirror {
	None = 0,
	LeftRight = 1,
	FrontBack = 2,
}

/// # Special Block Rotation (Packet Part)
/// The rotation state of a special block
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Structure_Block <br>
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Test_Instance_Block_Action
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum SpecialBlockRotation {
	None = 0,
	Clockwise90 = 1,
	Clockwise180 = 2,
	CounterClockwise90 = 3,
}

/// # Test Block Mode (Packet Part)
/// The mode of the test block
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Test_Block
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum TestBlockMode {
	Start = 0,
	Log = 1,
	Fail = 2,
	Accept = 3,
}

/// # Test Instance Block State (Packet Part)
/// Describes the current attempted action for a Test Instance Block
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Test_Instance_Block_Action
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum TestInstanceBlockActionAction {
	Init = 0,
	Query = 1,
	Set = 2,
	Reset = 3,
	Save = 4,
	Export = 5,
	Run = 6,
}

/// # Test Instance Status (Packet Part)
/// The status of the test instance.
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Test_Instance_Block_Action
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum TestInstanceStatus {
	Cleared = 0,
	Running = 1,
	Finished = 2,
}

/// # Block Face - Byte Enum (Packet Part)
/// The face of a block
///
/// As seen here: https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Action
#[derive(TypeEnum, McDefault, Debug, Clone, PartialEq)]
#[type_enum(i8)]
pub enum BlockFace {
	Bottom = 0,
	Top = 1,
	North = 2,
	South = 3,
	West = 4,
	East = 5,
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
	fn exact_property_sends_one_value() {
		let p = BlockProperty { name: "a".into(), is_exact: true, exact_value: Some("b".into()), min_value: None, max_value: None };
		assert_eq!(ser(&p), vec![1, b'a', 1, 1, b'b']);
		round_trip(p);
	}

	#[test]
	fn range_property_sends_min_and_max() {
		let p = BlockProperty { name: "a".into(), is_exact: false, exact_value: None, min_value: Some("1".into()), max_value: Some("3".into()) };
		assert_eq!(ser(&p), vec![1, b'a', 0, 1, b'1', 1, b'3']);
		round_trip(p);
	}

	#[test]
	fn empty_predicate_is_three_absent_flags() {
		let p = BlockPredicate::mc_default();
		assert_eq!(ser(&p), vec![0, 0, 0]);
		round_trip(p);
	}

	#[test]
	fn predicate_with_properties_round_trips() {
		round_trip(BlockPredicate {
			blocks: PrefixedOptional::new(None),
			properties: PrefixedOptional::new(Some(PrefixedArray::new(vec![BlockProperty::mc_default()]))),
			nbt: PrefixedOptional::new(None),
		});
	}

	#[test]
	fn command_block_flag_bits_match_wiki() {
		// Wiki (Update Command Block): 0x01 track output, 0x02 conditional, 0x04 automatic.
		assert_eq!(ser(&CommandBlockFlag::new(true, false, false)), vec![0x01]);
		assert_eq!(ser(&CommandBlockFlag::new(false, true, false)), vec![0x02]);
		assert_eq!(ser(&CommandBlockFlag::new(false, false, true)), vec![0x04]);
	}

	#[test]
	fn block_face_is_one_byte() {
		// Player Action and Use Item On send the face as a Byte or VarInt in range 0..=5.
		assert_eq!(ser(&BlockFace::Bottom), vec![0]);
		assert_eq!(ser(&BlockFace::East), vec![5]);
	}
}
