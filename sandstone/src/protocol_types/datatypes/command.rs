use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McSerialize};
use std::hash::{Hash, Hasher};

/// A Node used for representing graphs
#[derive(McSerialize, McDefault, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Node {
	pub flags: NodeFlags,
	pub children_count: VarInt,
	pub children: Vec<VarInt>,
	pub redirect_node: Option<VarInt>,
	pub name: Option<String>,
	pub parser: Option<Parser>,
	pub suggestions: Option<String>
}

impl McDeserialize for Node {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized{
		let flags = NodeFlags::mc_deserialize(deserializer)?;
		let children_count = VarInt::mc_deserialize(deserializer)?;

		let mut children = Vec::new();
		for _ in 0..children_count.0 {
			children.push(VarInt::mc_deserialize(deserializer)?);
		}

		let redirect_node = if flags.has_redirect {
			Some(VarInt::mc_deserialize(deserializer)?)
		} else {
			None
		};

		let name = if matches!(flags.typ, NodeType::Literal | NodeType::Argument) {
			Some(String::mc_deserialize(deserializer)?)
		} else {
			None
		};

		let parser = if matches!(flags.typ, NodeType::Argument) {
			Some(Parser::mc_deserialize(deserializer)?)
		} else {
			None
		};

		let suggestions = if flags.has_suggestions {
			Some(String::mc_deserialize(deserializer)?)
		} else {
			None
		};

		Ok(Self {
			flags,
			children_count,
			children,
			redirect_node,
			name,
			parser,
			suggestions
		})
	}
}

/// Internal node flags represented as a byte with masking
#[derive(McDefault, Debug, Clone, Hash, PartialEq, Eq)]
pub struct NodeFlags {
	pub typ: NodeType,
	pub is_executable: bool,
	pub has_redirect: bool,
	pub has_suggestions: bool,
	pub is_restricted: bool
}

impl NodeFlags {
	pub fn from_byte<'a>(byte: u8) -> SerializingResult<'a, NodeFlags> {
		Ok(Self {
			typ: match byte & 0x03 {
				0 => NodeType::Root,
				1 => NodeType::Literal,
				2 => NodeType::Argument,
				_ => {
					return Err(SerializingErr::InvalidEnumValue((byte & 0x03) as i8));
				}
			},
			is_executable: (byte & 0x04) != 0,
			has_redirect: (byte & 0x08) != 0,
			has_suggestions: (byte & 0x10) != 0,
			is_restricted: (byte & 0x20) != 0
		})
	}

	pub fn to_byte(&self) -> u8 {
		let mut byte = match self.typ {
			NodeType::Root => 0,
			NodeType::Literal => 1,
			NodeType::Argument => 2
		};
		if self.is_executable {
			byte |= 0x04;
		}
		if self.has_redirect {
			byte |= 0x08;
		}
		if self.has_suggestions {
			byte |= 0x10;
		}
		if self.is_restricted {
			byte |= 0x20;
		}
		byte
	}
}

impl McSerialize for NodeFlags {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.to_byte().mc_serialize(serializer)
	}
}

impl McDeserialize for NodeFlags {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		Self::from_byte(u8::mc_deserialize(deserializer)?)
	}
}

/// Type of node in a graph
#[derive(McDefault, Debug, Clone, Hash, PartialEq, Eq)]
pub enum NodeType {
	Root = 0,
	Literal = 1,
	Argument = 2,
}

#[derive(Debug, Clone)]
pub enum Parser {
	BrigadierBool,
	BrigadierFloat { flags: u8, min: Option<f32>, max: Option<f32> },
	BrigadierDouble { flags: u8, min: Option<f64>, max: Option<f64> },
	BrigadierInteger { flags: u8, min: Option<i32>, max: Option<i32> },
	BrigadierLong { flags: u8, min: Option<i64>, max: Option<i64> },
	BrigadierString(VarInt),
	MinecraftEntity(u8),
	MinecraftGameProfile,
	MinecraftBlockPos,
	MinecraftColumnPos,
	MinecraftVec3,
	MinecraftVec2,
	MinecraftBlockState,
	MinecraftBlockPredicate,
	MinecraftItemStack,
	MinecraftItemPredicate,
	MinecraftColor,
	MinecraftHexColor,
	MinecraftComponent,
	MinecraftStyle,
	MinecraftMessage,
	MinecraftNbtCompoundTag,
	MinecraftNbtTag,
	MinecraftNbtPath,
	MinecraftObjective,
	MinecraftObjectiveCriteria,
	MinecraftOperation,
	MinecraftParticle,
	MinecraftAngle,
	MinecraftRotation,
	MinecraftScoreboardSlot,
	MinecraftScoreHolder(u8),
	MinecraftSwizzle,
	MinecraftTeam,
	MinecraftItemSlot,
	MinecraftItemSlots,
	MinecraftResourceLocation,
	MinecraftFunction,
	MinecraftEntityAnchor,
	MinecraftIntRange,
	MinecraftFloatRange,
	MinecraftDimension,
	MinecraftGamemode,
	MinecraftTime(i32),
	MinecraftResourceOrTag(String),
	MinecraftResourceOrTagKey(String),
	MinecraftResource(String),
	MinecraftResourceKey(String),
	MinecraftResourceSelector(String),
	MinecraftTemplateMirror,
	MinecraftTemplateRotation,
	MinecraftHeightmap,
	MinecraftLootTable,
	MinecraftLootPredicate,
	MinecraftLootModifier,
	MinecraftDialog,
	MinecraftUuid,
}

fn deserialize_number_range_f32<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, (u8, Option<f32>, Option<f32>)> {
	let flags = u8::mc_deserialize(deserializer)?;
	let min = if flags & 0x01 != 0 { Some(f32::mc_deserialize(deserializer)?) } else { None };
	let max = if flags & 0x02 != 0 { Some(f32::mc_deserialize(deserializer)?) } else { None };
	Ok((flags, min, max))
}

fn serialize_number_range_f32(flags: u8, min: &Option<f32>, max: &Option<f32>, serializer: &mut McSerializer) -> SerializingResult<'static, ()> {
	flags.mc_serialize(serializer)?;
	if let Some(v) = min { v.mc_serialize(serializer)?; }
	if let Some(v) = max { v.mc_serialize(serializer)?; }
	Ok(())
}

fn deserialize_number_range_f64<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, (u8, Option<f64>, Option<f64>)> {
	let flags = u8::mc_deserialize(deserializer)?;
	let min = if flags & 0x01 != 0 { Some(f64::mc_deserialize(deserializer)?) } else { None };
	let max = if flags & 0x02 != 0 { Some(f64::mc_deserialize(deserializer)?) } else { None };
	Ok((flags, min, max))
}

fn serialize_number_range_f64(flags: u8, min: &Option<f64>, max: &Option<f64>, serializer: &mut McSerializer) -> SerializingResult<'static, ()> {
	flags.mc_serialize(serializer)?;
	if let Some(v) = min { v.mc_serialize(serializer)?; }
	if let Some(v) = max { v.mc_serialize(serializer)?; }
	Ok(())
}

fn deserialize_number_range_i32<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, (u8, Option<i32>, Option<i32>)> {
	let flags = u8::mc_deserialize(deserializer)?;
	let min = if flags & 0x01 != 0 { Some(i32::mc_deserialize(deserializer)?) } else { None };
	let max = if flags & 0x02 != 0 { Some(i32::mc_deserialize(deserializer)?) } else { None };
	Ok((flags, min, max))
}

fn serialize_number_range_i32(flags: u8, min: &Option<i32>, max: &Option<i32>, serializer: &mut McSerializer) -> SerializingResult<'static, ()> {
	flags.mc_serialize(serializer)?;
	if let Some(v) = min { v.mc_serialize(serializer)?; }
	if let Some(v) = max { v.mc_serialize(serializer)?; }
	Ok(())
}

fn deserialize_number_range_i64<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, (u8, Option<i64>, Option<i64>)> {
	let flags = u8::mc_deserialize(deserializer)?;
	let min = if flags & 0x01 != 0 { Some(i64::mc_deserialize(deserializer)?) } else { None };
	let max = if flags & 0x02 != 0 { Some(i64::mc_deserialize(deserializer)?) } else { None };
	Ok((flags, min, max))
}

fn serialize_number_range_i64(flags: u8, min: &Option<i64>, max: &Option<i64>, serializer: &mut McSerializer) -> SerializingResult<'static, ()> {
	flags.mc_serialize(serializer)?;
	if let Some(v) = min { v.mc_serialize(serializer)?; }
	if let Some(v) = max { v.mc_serialize(serializer)?; }
	Ok(())
}

impl McSerialize for Parser {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			Parser::BrigadierBool => VarInt(0).mc_serialize(serializer)?,
			Parser::BrigadierFloat { flags, min, max } => {
				VarInt(1).mc_serialize(serializer)?;
				serialize_number_range_f32(*flags, min, max, serializer)?;
			},
			Parser::BrigadierDouble { flags, min, max } => {
				VarInt(2).mc_serialize(serializer)?;
				serialize_number_range_f64(*flags, min, max, serializer)?;
			},
			Parser::BrigadierInteger { flags, min, max } => {
				VarInt(3).mc_serialize(serializer)?;
				serialize_number_range_i32(*flags, min, max, serializer)?;
			},
			Parser::BrigadierLong { flags, min, max } => {
				VarInt(4).mc_serialize(serializer)?;
				serialize_number_range_i64(*flags, min, max, serializer)?;
			},
			Parser::BrigadierString(behavior) => {
				VarInt(5).mc_serialize(serializer)?;
				behavior.mc_serialize(serializer)?;
			},
			Parser::MinecraftEntity(flags) => {
				VarInt(6).mc_serialize(serializer)?;
				flags.mc_serialize(serializer)?;
			},
			Parser::MinecraftGameProfile => VarInt(7).mc_serialize(serializer)?,
			Parser::MinecraftBlockPos => VarInt(8).mc_serialize(serializer)?,
			Parser::MinecraftColumnPos => VarInt(9).mc_serialize(serializer)?,
			Parser::MinecraftVec3 => VarInt(10).mc_serialize(serializer)?,
			Parser::MinecraftVec2 => VarInt(11).mc_serialize(serializer)?,
			Parser::MinecraftBlockState => VarInt(12).mc_serialize(serializer)?,
			Parser::MinecraftBlockPredicate => VarInt(13).mc_serialize(serializer)?,
			Parser::MinecraftItemStack => VarInt(14).mc_serialize(serializer)?,
			Parser::MinecraftItemPredicate => VarInt(15).mc_serialize(serializer)?,
			Parser::MinecraftColor => VarInt(16).mc_serialize(serializer)?,
			Parser::MinecraftHexColor => VarInt(17).mc_serialize(serializer)?,
			Parser::MinecraftComponent => VarInt(18).mc_serialize(serializer)?,
			Parser::MinecraftStyle => VarInt(19).mc_serialize(serializer)?,
			Parser::MinecraftMessage => VarInt(20).mc_serialize(serializer)?,
			Parser::MinecraftNbtCompoundTag => VarInt(21).mc_serialize(serializer)?,
			Parser::MinecraftNbtTag => VarInt(22).mc_serialize(serializer)?,
			Parser::MinecraftNbtPath => VarInt(23).mc_serialize(serializer)?,
			Parser::MinecraftObjective => VarInt(24).mc_serialize(serializer)?,
			Parser::MinecraftObjectiveCriteria => VarInt(25).mc_serialize(serializer)?,
			Parser::MinecraftOperation => VarInt(26).mc_serialize(serializer)?,
			Parser::MinecraftParticle => VarInt(27).mc_serialize(serializer)?,
			Parser::MinecraftAngle => VarInt(28).mc_serialize(serializer)?,
			Parser::MinecraftRotation => VarInt(29).mc_serialize(serializer)?,
			Parser::MinecraftScoreboardSlot => VarInt(30).mc_serialize(serializer)?,
			Parser::MinecraftScoreHolder(flags) => {
				VarInt(31).mc_serialize(serializer)?;
				flags.mc_serialize(serializer)?;
			},
			Parser::MinecraftSwizzle => VarInt(32).mc_serialize(serializer)?,
			Parser::MinecraftTeam => VarInt(33).mc_serialize(serializer)?,
			Parser::MinecraftItemSlot => VarInt(34).mc_serialize(serializer)?,
			Parser::MinecraftItemSlots => VarInt(35).mc_serialize(serializer)?,
			Parser::MinecraftResourceLocation => VarInt(36).mc_serialize(serializer)?,
			Parser::MinecraftFunction => VarInt(37).mc_serialize(serializer)?,
			Parser::MinecraftEntityAnchor => VarInt(38).mc_serialize(serializer)?,
			Parser::MinecraftIntRange => VarInt(39).mc_serialize(serializer)?,
			Parser::MinecraftFloatRange => VarInt(40).mc_serialize(serializer)?,
			Parser::MinecraftDimension => VarInt(41).mc_serialize(serializer)?,
			Parser::MinecraftGamemode => VarInt(42).mc_serialize(serializer)?,
			Parser::MinecraftTime(min) => {
				VarInt(43).mc_serialize(serializer)?;
				min.mc_serialize(serializer)?;
			},
			Parser::MinecraftResourceOrTag(registry) => {
				VarInt(44).mc_serialize(serializer)?;
				registry.mc_serialize(serializer)?;
			},
			Parser::MinecraftResourceOrTagKey(registry) => {
				VarInt(45).mc_serialize(serializer)?;
				registry.mc_serialize(serializer)?;
			},
			Parser::MinecraftResource(registry) => {
				VarInt(46).mc_serialize(serializer)?;
				registry.mc_serialize(serializer)?;
			},
			Parser::MinecraftResourceKey(registry) => {
				VarInt(47).mc_serialize(serializer)?;
				registry.mc_serialize(serializer)?;
			},
			Parser::MinecraftResourceSelector(registry) => {
				VarInt(48).mc_serialize(serializer)?;
				registry.mc_serialize(serializer)?;
			},
			Parser::MinecraftTemplateMirror => VarInt(49).mc_serialize(serializer)?,
			Parser::MinecraftTemplateRotation => VarInt(50).mc_serialize(serializer)?,
			Parser::MinecraftHeightmap => VarInt(51).mc_serialize(serializer)?,
			Parser::MinecraftLootTable => VarInt(52).mc_serialize(serializer)?,
			Parser::MinecraftLootPredicate => VarInt(53).mc_serialize(serializer)?,
			Parser::MinecraftLootModifier => VarInt(54).mc_serialize(serializer)?,
			Parser::MinecraftDialog => VarInt(55).mc_serialize(serializer)?,
			Parser::MinecraftUuid => VarInt(56).mc_serialize(serializer)?,
		}
		Ok(())
	}
}

impl McDeserialize for Parser {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let id = VarInt::mc_deserialize(deserializer)?.0;
		match id {
			0 => Ok(Parser::BrigadierBool),
			1 => {
				let (flags, min, max) = deserialize_number_range_f32(deserializer)?;
				Ok(Parser::BrigadierFloat { flags, min, max })
			},
			2 => {
				let (flags, min, max) = deserialize_number_range_f64(deserializer)?;
				Ok(Parser::BrigadierDouble { flags, min, max })
			},
			3 => {
				let (flags, min, max) = deserialize_number_range_i32(deserializer)?;
				Ok(Parser::BrigadierInteger { flags, min, max })
			},
			4 => {
				let (flags, min, max) = deserialize_number_range_i64(deserializer)?;
				Ok(Parser::BrigadierLong { flags, min, max })
			},
			5 => Ok(Parser::BrigadierString(VarInt::mc_deserialize(deserializer)?)),
			6 => Ok(Parser::MinecraftEntity(u8::mc_deserialize(deserializer)?)),
			7 => Ok(Parser::MinecraftGameProfile),
			8 => Ok(Parser::MinecraftBlockPos),
			9 => Ok(Parser::MinecraftColumnPos),
			10 => Ok(Parser::MinecraftVec3),
			11 => Ok(Parser::MinecraftVec2),
			12 => Ok(Parser::MinecraftBlockState),
			13 => Ok(Parser::MinecraftBlockPredicate),
			14 => Ok(Parser::MinecraftItemStack),
			15 => Ok(Parser::MinecraftItemPredicate),
			16 => Ok(Parser::MinecraftColor),
			17 => Ok(Parser::MinecraftHexColor),
			18 => Ok(Parser::MinecraftComponent),
			19 => Ok(Parser::MinecraftStyle),
			20 => Ok(Parser::MinecraftMessage),
			21 => Ok(Parser::MinecraftNbtCompoundTag),
			22 => Ok(Parser::MinecraftNbtTag),
			23 => Ok(Parser::MinecraftNbtPath),
			24 => Ok(Parser::MinecraftObjective),
			25 => Ok(Parser::MinecraftObjectiveCriteria),
			26 => Ok(Parser::MinecraftOperation),
			27 => Ok(Parser::MinecraftParticle),
			28 => Ok(Parser::MinecraftAngle),
			29 => Ok(Parser::MinecraftRotation),
			30 => Ok(Parser::MinecraftScoreboardSlot),
			31 => Ok(Parser::MinecraftScoreHolder(u8::mc_deserialize(deserializer)?)),
			32 => Ok(Parser::MinecraftSwizzle),
			33 => Ok(Parser::MinecraftTeam),
			34 => Ok(Parser::MinecraftItemSlot),
			35 => Ok(Parser::MinecraftItemSlots),
			36 => Ok(Parser::MinecraftResourceLocation),
			37 => Ok(Parser::MinecraftFunction),
			38 => Ok(Parser::MinecraftEntityAnchor),
			39 => Ok(Parser::MinecraftIntRange),
			40 => Ok(Parser::MinecraftFloatRange),
			41 => Ok(Parser::MinecraftDimension),
			42 => Ok(Parser::MinecraftGamemode),
			43 => Ok(Parser::MinecraftTime(i32::mc_deserialize(deserializer)?)),
			44 => Ok(Parser::MinecraftResourceOrTag(String::mc_deserialize(deserializer)?)),
			45 => Ok(Parser::MinecraftResourceOrTagKey(String::mc_deserialize(deserializer)?)),
			46 => Ok(Parser::MinecraftResource(String::mc_deserialize(deserializer)?)),
			47 => Ok(Parser::MinecraftResourceKey(String::mc_deserialize(deserializer)?)),
			48 => Ok(Parser::MinecraftResourceSelector(String::mc_deserialize(deserializer)?)),
			49 => Ok(Parser::MinecraftTemplateMirror),
			50 => Ok(Parser::MinecraftTemplateRotation),
			51 => Ok(Parser::MinecraftHeightmap),
			52 => Ok(Parser::MinecraftLootTable),
			53 => Ok(Parser::MinecraftLootPredicate),
			54 => Ok(Parser::MinecraftLootModifier),
			55 => Ok(Parser::MinecraftDialog),
			56 => Ok(Parser::MinecraftUuid),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid parser ID: '{}'", id)))
		}
	}
}

impl McDefault for Parser {
	fn mc_default() -> Self {
		Parser::BrigadierBool
	}
}

impl Hash for Parser {
	fn hash<H: Hasher>(&self, state: &mut H) {
		core::mem::discriminant(self).hash(state);
		match self {
			Parser::BrigadierFloat { flags, min, max } => {
				flags.hash(state);
				min.map(|v| v.to_bits()).hash(state);
				max.map(|v| v.to_bits()).hash(state);
			},
			Parser::BrigadierDouble { flags, min, max } => {
				flags.hash(state);
				min.map(|v| v.to_bits()).hash(state);
				max.map(|v| v.to_bits()).hash(state);
			},
			Parser::BrigadierInteger { flags, min, max } => {
				flags.hash(state);
				min.hash(state);
				max.hash(state);
			},
			Parser::BrigadierLong { flags, min, max } => {
				flags.hash(state);
				min.hash(state);
				max.hash(state);
			},
			Parser::BrigadierString(v) => v.hash(state),
			Parser::MinecraftEntity(v) => v.hash(state),
			Parser::MinecraftScoreHolder(v) => v.hash(state),
			Parser::MinecraftTime(v) => v.hash(state),
			Parser::MinecraftResourceOrTag(v) => v.hash(state),
			Parser::MinecraftResourceOrTagKey(v) => v.hash(state),
			Parser::MinecraftResource(v) => v.hash(state),
			Parser::MinecraftResourceKey(v) => v.hash(state),
			Parser::MinecraftResourceSelector(v) => v.hash(state),
			_ => {}
		}
	}
}

impl PartialEq for Parser {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Parser::BrigadierFloat { flags: f1, min: mn1, max: mx1 }, Parser::BrigadierFloat { flags: f2, min: mn2, max: mx2 }) => {
				f1 == f2 && mn1.map(|v| v.to_bits()) == mn2.map(|v| v.to_bits()) && mx1.map(|v| v.to_bits()) == mx2.map(|v| v.to_bits())
			},
			(Parser::BrigadierDouble { flags: f1, min: mn1, max: mx1 }, Parser::BrigadierDouble { flags: f2, min: mn2, max: mx2 }) => {
				f1 == f2 && mn1.map(|v| v.to_bits()) == mn2.map(|v| v.to_bits()) && mx1.map(|v| v.to_bits()) == mx2.map(|v| v.to_bits())
			},
			(Parser::BrigadierInteger { flags: f1, min: mn1, max: mx1 }, Parser::BrigadierInteger { flags: f2, min: mn2, max: mx2 }) => {
				f1 == f2 && mn1 == mn2 && mx1 == mx2
			},
			(Parser::BrigadierLong { flags: f1, min: mn1, max: mx1 }, Parser::BrigadierLong { flags: f2, min: mn2, max: mx2 }) => {
				f1 == f2 && mn1 == mn2 && mx1 == mx2
			},
			(Parser::BrigadierString(a), Parser::BrigadierString(b)) => a == b,
			(Parser::MinecraftEntity(a), Parser::MinecraftEntity(b)) => a == b,
			(Parser::MinecraftScoreHolder(a), Parser::MinecraftScoreHolder(b)) => a == b,
			(Parser::MinecraftTime(a), Parser::MinecraftTime(b)) => a == b,
			(Parser::MinecraftResourceOrTag(a), Parser::MinecraftResourceOrTag(b)) => a == b,
			(Parser::MinecraftResourceOrTagKey(a), Parser::MinecraftResourceOrTagKey(b)) => a == b,
			(Parser::MinecraftResource(a), Parser::MinecraftResource(b)) => a == b,
			(Parser::MinecraftResourceKey(a), Parser::MinecraftResourceKey(b)) => a == b,
			(Parser::MinecraftResourceSelector(a), Parser::MinecraftResourceSelector(b)) => a == b,
			_ => core::mem::discriminant(self) == core::mem::discriminant(other)
		}
	}
}

impl Eq for Parser {}