use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

#[derive(McDefault, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub enum SlotDisplay {
	Empty = 0,
	AnyFuel = 1,
	Item(VarInt) = 2,
	ItemStack = 3, //todo: Slot datatype
	Tag(String) = 4,
	SmithingingTrim(Box<SmithingTrimSlotData>) = 5,
	WithRemainder(Box<WithRemainderSlotData>) = 6,
	Composite(Box<CompositeSlotData>) = 7
}

// https://minecraft.wiki/w/Java_Edition_protocol/Recipes#Slot_Display_structure
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Recipes

impl McSerialize for SlotDisplay {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {

		match self {
			SlotDisplay::Empty => 0i8,
			SlotDisplay::AnyFuel => 1i8,
			SlotDisplay::Item(id) => {
				id.mc_serialize(serializer)?;
				2i8
			},
			SlotDisplay::ItemStack => 3i8,
			SlotDisplay::Tag(tag) => {
				tag.mc_serialize(serializer)?;
				4i8
			},
			SlotDisplay::SmithingingTrim(data) => {
				data.mc_serialize(serializer)?;
				5i8
			},
			SlotDisplay::WithRemainder(data) => {
				data.mc_serialize(serializer)?;
				6i8
			},
			SlotDisplay::Composite(data) => {
				7i8.mc_serialize(serializer)?;
				data.mc_serialize(serializer)?;
			}
		}

		Ok(())
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct SmithingTrimSlotData {
	pub base: SlotDisplay,
	pub material: SlotDisplay,
	pub pattern: VarInt
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct WithRemainderSlotData {
	pub ingredient: SlotDisplay,
	pub remainder: SlotDisplay
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct CompositeSlotData {
	pub option_count: VarInt,
	pub options: Vec<SlotDisplay>
}

// TODO: https://minecraft.wiki/w/Java_Edition_protocol/Slot_data