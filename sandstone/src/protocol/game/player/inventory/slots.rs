use crate::protocol::game::player::inventory::components::StructuredComponent;
use crate::protocol::game::player::inventory::slotdata::SlotData;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
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
	fn slot_display_ids_match_recipe_wiki() {
		// The client picks the display type from this VarInt, so a wrong ID shows the wrong ingredient.
		assert_eq!(ser(&SlotDisplay::Empty), vec![0]);
		assert_eq!(ser(&SlotDisplay::AnyFuel), vec![1]);
		assert_eq!(ser(&SlotDisplay::Item(VarInt(7))), vec![2, 7]);
		assert_eq!(ser(&SlotDisplay::Tag("a".into())), vec![4, 1, b'a']);
	}

	#[test]
	fn nested_slot_display_round_trips() {
		round_trip(SlotDisplay::Composite(Box::new(CompositeSlotData {
			options: PrefixedArray::new(vec![
				SlotDisplay::Item(VarInt(1)),
				SlotDisplay::WithRemainder(Box::new(WithRemainderSlotData { ingredient: SlotDisplay::AnyFuel, remainder: SlotDisplay::Empty })),
			]),
		})));
	}

	#[test]
	fn empty_hashed_slot_is_one_byte() {
		// A slot with no item sends only `has_item = false`. Extra bytes break the click packet.
		let slot = HashedSlot { has_item: false, item_id: None, item_count: None, components: None, components_to_remove: None };
		assert_eq!(ser(&slot), vec![0]);
		round_trip(slot);
	}

	#[test]
	fn full_hashed_slot_round_trips() {
		round_trip(ChangedSlot {
			slot: 36,
			slot_data: HashedSlot {
				has_item: true,
				item_id: Some(VarInt(1)),
				item_count: Some(VarInt(64)),
				components: Some(PrefixedArray::new(vec![ComponentHashed { component_type: StructuredComponent::Unbreakable, data_hash: -5 }])),
				components_to_remove: Some(PrefixedArray::new(vec![])),
			},
		});
	}

	#[test]
	fn inventory_mode_ids() {
		assert_eq!(ser(&InventoryOperationMode::Pickup), vec![0]);
		assert_eq!(ser(&InventoryOperationMode::Drag), vec![5]);
		assert_eq!(ser(&InventoryOperationMode::PickupAll), vec![6]);
	}
}
