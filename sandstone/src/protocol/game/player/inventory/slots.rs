use crate::protocol::game::player::inventory::components::StructuredComponent;
use crate::protocol::game::player::inventory::slotdata::SlotData;
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

/// Description of a recipe ingredient slot for use for use by the client.
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Recipes#Slot_Display_structure
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

/// # Changed Slot (Packet Part)
/// New data for a slot that the client wants to inform the server about.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ChangedSlot {
	pub slot: i16,
	/// New data for this slot, in the client's opinion. Server verifies this data.
	pub slot_data: HashedSlot
}

/// # Hashed Slot (Packet Part)
/// Used to communicate slot changes in an inventory.
///
/// https://minecraft.wiki/w/Java_Edition_protocol/Slot_data#Hashed_Format
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct HashedSlot {
	pub has_item: bool,
	#[mc(deserialize_if = has_item)]
	pub item_id: Option<VarInt>,
	#[mc(deserialize_if = has_item)]
	pub item_count: Option<VarInt>,
	#[mc(deserialize_if = has_item)]
	pub components: Option<PrefixedArray<ComponentHashed>>,
	#[mc(deserialize_if = has_item)]
	pub components_to_remove: Option<PrefixedArray<StructuredComponent>>
}

/// # Slot Data Hash (Packet Part)
/// Used for inventory updates.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ComponentHashed {
	pub component_type: StructuredComponent,
	/// A CRC32C (note: CRC32C is not the same thing as CRC32) checksum of the component data. Currently undocumented
	pub data_hash: i32
}

// TODO: Map valid mode and button combinations?
/// # Inventory Operation Mode (Packet Part)
/// Used to determine valid buttons and operations for inventory actions.
#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
pub enum InventoryOperationMode {
	Pickup = 0,
	ShiftClick = 1,
	NumberKeySwap = 2,
	MiddleClick = 3,
	Drop = 4,
	Drag = 5,
	PickupAll = 6
}
