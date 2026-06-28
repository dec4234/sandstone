use crate::bitflag;
use crate::protocol::game::effects::particle::Particle;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::internal_types::IDSet;
use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

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

/// # Test Instance Status
/// The status of the test instance.
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Test_Instance_Block_Action
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum TestInstanceStatus {
	Cleared = 0,
	Running = 1,
	Finished = 2,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
#[repr(i8)]
pub enum BlockFace {
	Bottom = 0,
	Top = 1,
	North = 2,
	South = 3,
	West = 4,
	East = 5,
}
