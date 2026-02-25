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
	Empty,
	AnyFuel,
	Item(VarInt),
	ItemStack, //todo: SlotData datatype
	Tag(String),
	SmithingingTrim(Box<SmithingTrimSlotData>),
	WithRemainder(Box<WithRemainderSlotData>),
	Composite(Box<CompositeSlotData>)
}

// https://minecraft.wiki/w/Java_Edition_protocol/Recipes#Slot_Display_structure
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Recipes

impl McSerialize for SlotDisplay {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {

		match self {
			SlotDisplay::Empty => {
				 0i8.mc_serialize(serializer)?;
			},
			SlotDisplay::AnyFuel => {
				1i8.mc_serialize(serializer)?;
			},
			SlotDisplay::Item(id) => {
				2i8.mc_serialize(serializer)?;
				id.mc_serialize(serializer)?;
			},
			SlotDisplay::ItemStack => {
				3i8.mc_serialize(serializer)?;
			},
			SlotDisplay::Tag(tag) => {
				4i8.mc_serialize(serializer)?;
				tag.mc_serialize(serializer)?;

			},
			SlotDisplay::SmithingingTrim(data) => {
				5i8.mc_serialize(serializer)?;
				data.mc_serialize(serializer)?;
			},
			SlotDisplay::WithRemainder(data) => {
				6i8.mc_serialize(serializer)?;
				data.mc_serialize(serializer)?;
			},
			SlotDisplay::Composite(data) => {
				7i8.mc_serialize(serializer)?;
				data.mc_serialize(serializer)?;
			}
		}

		Ok(())
	}
}

impl McDeserialize for SlotDisplay {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = i8::mc_deserialize(deserializer)?;
		match typ {
			0 => Ok(SlotDisplay::Empty),
			1 => Ok(SlotDisplay::AnyFuel),
			2 => Ok(SlotDisplay::Item(VarInt::mc_deserialize(deserializer)?)),
			3 => Ok(SlotDisplay::ItemStack),
			4 => Ok(SlotDisplay::Tag(String::mc_deserialize(deserializer)?)),
			5 => Ok(SlotDisplay::SmithingingTrim(Box::new(SmithingTrimSlotData::mc_deserialize(deserializer)?))),
			6 => Ok(SlotDisplay::WithRemainder(Box::new(WithRemainderSlotData::mc_deserialize(deserializer)?))),
			7 => Ok(SlotDisplay::Composite(Box::new(CompositeSlotData::mc_deserialize(deserializer)?))),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid SlotDisplay type: {}", typ)))
		}
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
#[derive(McDefault, McSerialize, McDeserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct SlotData {
	pub item_count: VarInt,
	pub item_id: Option<VarInt>,
	pub components_to_add: Option<VarInt>,
	pub components_to_remove: Option<VarInt>,
	// todo: lot of work for the enum
}