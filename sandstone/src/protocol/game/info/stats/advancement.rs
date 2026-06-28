use crate::bitflag;
use crate::protocol::game::player::inventory::slotdata::SlotData;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::{McDeserialize, McSerialize};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, VarIntEnum};
use sandstone_derive::{McDeserialize, McSerialize};
use std::fmt::Debug;

bitflag!(AdvancementFlags: i32 {
	has_background_texture, show_toast, hidden
});

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
	#[mc(deserialize_if = flags.has_background_texture())]
	pub background_texture: Option<String>,
	pub x: f32,
	pub y: f32,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum AdvancementFrameType {
	Task = 0,
	Challenge = 1,
	Goal = 2,
}
