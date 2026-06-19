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
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

// https://minecraft.wiki/w/Java_Edition_protocol/Recipes#Slot_Display_structure
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Recipes

#[derive(VarIntEnum, McDefault, Debug, PartialEq, Clone)]
#[repr(i32)]
pub enum SlotDisplay {
	Empty = 0,
	AnyFuel = 1,
	Item(VarInt) = 2,
	ItemStack(SlotData) = 3,
	Tag(String) = 4,
	SmithingingTrim(Box<SmithingTrimSlotData>) = 5,
	WithRemainder(Box<WithRemainderSlotData>) = 6,
	Composite(Box<CompositeSlotData>) = 7,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, PartialEq, Clone)]
pub struct SmithingTrimSlotData {
	pub base: SlotDisplay,
	pub material: SlotDisplay,
	pub pattern: VarInt,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, PartialEq, Clone)]
pub struct WithRemainderSlotData {
	pub ingredient: SlotDisplay,
	pub remainder: SlotDisplay,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, PartialEq, Clone)]
pub struct CompositeSlotData {
	pub options: PrefixedArray<SlotDisplay>,
}

/// https://minecraft.wiki/w/Java_Edition_protocol/Recipes#Recipe_Display
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum RecipeDisplay {
	CraftingShapeless(CraftingShapelessDisplay) = 0,
	CraftingShaped(CraftingShapedDisplay) = 1,
	Furnace(FurnaceDisplay) = 2,
	Stonecutter(StonecutterDisplay) = 3,
	Smithing(SmithingDisplay) = 4,
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
