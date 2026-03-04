use crate::protocol::game::info::inventory::slotdata::SlotData;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
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
	ItemStack(SlotData),
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
				 VarInt(0).mc_serialize(serializer)?;
			},
			SlotDisplay::AnyFuel => {
				VarInt(1).mc_serialize(serializer)?;
			},
			SlotDisplay::Item(id) => {
				VarInt(2).mc_serialize(serializer)?;
				id.mc_serialize(serializer)?;
			},
			SlotDisplay::ItemStack(data) => {
				VarInt(3).mc_serialize(serializer)?;
				data.mc_serialize(serializer)?;
			},
			SlotDisplay::Tag(tag) => {
				VarInt(4).mc_serialize(serializer)?;
				tag.mc_serialize(serializer)?;

			},
			SlotDisplay::SmithingingTrim(data) => {
				VarInt(5).mc_serialize(serializer)?;
				data.mc_serialize(serializer)?;
			},
			SlotDisplay::WithRemainder(data) => {
				VarInt(6).mc_serialize(serializer)?;
				data.mc_serialize(serializer)?;
			},
			SlotDisplay::Composite(data) => {
				VarInt(7).mc_serialize(serializer)?;
				data.mc_serialize(serializer)?;
			}
		}

		Ok(())
	}
}

impl McDeserialize for SlotDisplay {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		match typ {
			0 => Ok(SlotDisplay::Empty),
			1 => Ok(SlotDisplay::AnyFuel),
			2 => Ok(SlotDisplay::Item(VarInt::mc_deserialize(deserializer)?)),
			3 => Ok(SlotDisplay::ItemStack(SlotData::mc_deserialize(deserializer)?)),
			4 => Ok(SlotDisplay::Tag(String::mc_deserialize(deserializer)?)),
			5 => Ok(SlotDisplay::SmithingingTrim(Box::new(SmithingTrimSlotData::mc_deserialize(deserializer)?))),
			6 => Ok(SlotDisplay::WithRemainder(Box::new(WithRemainderSlotData::mc_deserialize(deserializer)?))),
			7 => Ok(SlotDisplay::Composite(Box::new(CompositeSlotData::mc_deserialize(deserializer)?))),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid SlotDisplay type: '{}'", typ)))
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
	pub options: PrefixedArray<SlotDisplay>
}

/// https://minecraft.wiki/w/Java_Edition_protocol/Recipes#Recipe_Display
#[derive(McDefault, Debug, Clone, PartialEq)]
pub enum RecipeDisplay {
	CraftingShapeless(CraftingShapelessDisplay),
	CraftingShaped(CraftingShapedDisplay),
	Furnace(FurnaceDisplay),
	Stonecutter(StonecutterDisplay),
	Smithing(SmithingDisplay),
}

impl McSerialize for RecipeDisplay {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			RecipeDisplay::CraftingShapeless(d) => { VarInt(0).mc_serialize(serializer)?; d.mc_serialize(serializer)?; },
			RecipeDisplay::CraftingShaped(d) => { VarInt(1).mc_serialize(serializer)?; d.mc_serialize(serializer)?; },
			RecipeDisplay::Furnace(d) => { VarInt(2).mc_serialize(serializer)?; d.mc_serialize(serializer)?; },
			RecipeDisplay::Stonecutter(d) => { VarInt(3).mc_serialize(serializer)?; d.mc_serialize(serializer)?; },
			RecipeDisplay::Smithing(d) => { VarInt(4).mc_serialize(serializer)?; d.mc_serialize(serializer)?; },
		}
		Ok(())
	}
}

impl McDeserialize for RecipeDisplay {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		match typ {
			0 => Ok(RecipeDisplay::CraftingShapeless(CraftingShapelessDisplay::mc_deserialize(deserializer)?)),
			1 => Ok(RecipeDisplay::CraftingShaped(CraftingShapedDisplay::mc_deserialize(deserializer)?)),
			2 => Ok(RecipeDisplay::Furnace(FurnaceDisplay::mc_deserialize(deserializer)?)),
			3 => Ok(RecipeDisplay::Stonecutter(StonecutterDisplay::mc_deserialize(deserializer)?)),
			4 => Ok(RecipeDisplay::Smithing(SmithingDisplay::mc_deserialize(deserializer)?)),
			_ => Err(SerializingErr::DeserializationError(format!("Invalid RecipeDisplay type: '{}'", typ)))
		}
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct CraftingShapelessDisplay {
	pub ingredients: PrefixedArray<SlotDisplay>,
	pub result: SlotDisplay,
	pub crafting_station: SlotDisplay,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct CraftingShapedDisplay {
	pub width: VarInt,
	pub height: VarInt,
	pub ingredients: PrefixedArray<SlotDisplay>,
	pub result: SlotDisplay,
	pub crafting_station: SlotDisplay,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct FurnaceDisplay {
	pub ingredient: SlotDisplay,
	pub fuel: SlotDisplay,
	pub result: SlotDisplay,
	pub duration: VarInt,
	pub experience: f32,
	pub crafting_station: SlotDisplay,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct StonecutterDisplay {
	pub input: SlotDisplay,
	pub result: SlotDisplay,
	pub crafting_station: SlotDisplay,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct SmithingDisplay {
	pub template: SlotDisplay,
	pub base: SlotDisplay,
	pub addition: SlotDisplay,
	pub result: SlotDisplay,
	pub crafting_station: SlotDisplay,
}