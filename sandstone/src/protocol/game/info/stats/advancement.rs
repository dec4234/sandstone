use crate::protocol::game::info::inventory::slotdata::SlotData;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::{McDeserialize, McSerialize};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::McDefault;
use sandstone_derive::{McDeserialize, McSerialize};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub struct Mapping<T> {
	pub key: String,
	pub value: T,
}

impl<T: McSerialize> McSerialize for Mapping<T> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.key.mc_serialize(serializer)?;
		self.value.mc_serialize(serializer)
	}
}

impl<T: McDeserialize> McDeserialize for Mapping<T> {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let key = String::mc_deserialize(deserializer)?;
		let value = T::mc_deserialize(deserializer)?;
		Ok(Self { key, value })
	}
}

impl<T: McDefault> McDefault for Mapping<T> {
	fn mc_default() -> Self {
		Self { key: McDefault::mc_default(), value: T::mc_default() }
	}
}

#[derive(McSerialize, McDeserialize, McDefault, Debug, Clone, PartialEq)]
pub struct Advancement {
	pub parent_id: PrefixedOptional<String>,
	pub display_data: PrefixedOptional<AdvancementDisplay>,
	pub nested_requirements: PrefixedArray<PrefixedArray<String>>,
	pub sends_telemetry_data: bool,
}

#[derive(McSerialize, McDeserialize, McDefault, Debug, Clone, PartialEq)]
pub struct AdvancementDisplay {
	pub title: TextComponent,
	pub description: TextComponent,
	pub icon: SlotData,
	pub frame_type: AdvancementFrameType,
	pub flags: AdvancementFlags,
	#[mc(deserialize_if = flags.has_background_texture)]
	pub background_texture: Option<String>,
	pub x: f32,
	pub y: f32,
}

#[derive(McDefault, Debug, Clone, PartialEq)]
pub enum AdvancementFrameType {
	Task,
	Challenge,
	Goal
}

impl McSerialize for AdvancementFrameType {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let value = match self {
			AdvancementFrameType::Task => 0,
			AdvancementFrameType::Challenge => 1,
			AdvancementFrameType::Goal => 2
		};
		VarInt(value).mc_serialize(serializer)
	}
}

impl McDeserialize for AdvancementFrameType {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let value = VarInt::mc_deserialize(deserializer)?.0;
		let frame_type = match value {
			0 => AdvancementFrameType::Task,
			1 => AdvancementFrameType::Challenge,
			2 => AdvancementFrameType::Goal,
			_ => return Err(SerializingErr::OutOfBounds(format!("Invalid advancement frame type value {}: must be 0, 1, or 2", value)))
		};
		Ok(frame_type)
	}
}

#[derive(McDefault, Debug, Clone, PartialEq)]
pub struct AdvancementFlags {
	pub has_background_texture: bool,
	pub show_toast: bool,
	pub hidden: bool,
}

impl McSerialize for AdvancementFlags {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let mut flags = 0;
		if self.has_background_texture {
			flags |= 0x01;
		}
		if self.show_toast {
			flags |= 0x02;
		}
		if self.hidden {
			flags |= 0x04;
		}
		flags.mc_serialize(serializer)
	}
}

impl McDeserialize for AdvancementFlags {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let flags = VarInt::mc_deserialize(deserializer)?.0;
		Ok(Self {
			has_background_texture: (flags & 0x01) != 0,
			show_toast: (flags & 0x02) != 0,
			hidden: (flags & 0x04) != 0,
		})
	}
}

#[derive(McSerialize, McDeserialize, McDefault, Debug, Clone, PartialEq)]
pub struct AdvancementProgress {
	pub criterion: String,
	pub date_acheived: PrefixedOptional<i64>
}