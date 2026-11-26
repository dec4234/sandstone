use crate::protocol::game::player::inventory::components::StructuredComponent;
use crate::protocol::game::player::inventory::slotdata::ItemStackTemplate;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::internal_types::IDSet;
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
	WithAnyPotion(Box<SlotDisplay>) = 2,
	OnlyWithComponent(Box<OnlyWithComponentSlotData>) = 3,
	Item(VarInt) = 4,
	/// The wiki shows a Slot here, but a 26.3 server sends an item-first template.
	ItemStack(ItemStackTemplate) = 5,
	/// The wiki shows an Identifier here, but a 26.3 server sends an ID Set (type 0 + tag name).
	Tag(IDSet) = 6,
	Dyed(Box<DyedSlotData>) = 7,
	SmithingingTrim(Box<SmithingTrimSlotData>) = 8,
	WithRemainder(Box<WithRemainderSlotData>) = 9,
	Composite(Box<CompositeSlotData>) = 10,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, PartialEq, Clone)]
pub struct OnlyWithComponentSlotData {
	pub base: SlotDisplay,
	/// ID in the `minecraft:data_component_type` registry.
	pub component_type: VarInt,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, PartialEq, Clone)]
pub struct DyedSlotData {
	pub dye: SlotDisplay,
	pub target: SlotDisplay,
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

/// https://minecraft.wiki/w/Java_Edition_protocol/Recipes#Recipe_Display_structure
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
	pub crafting_station: SlotDisplay,
	pub duration: VarInt,
	pub experience: f32,
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
/// https://minecraft.wiki/w/Java_Edition_protocol/Slot_data#Hashed_format
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
#[derive(VarIntEnum, McDefault, Debug, Clone, Copy, PartialEq)]
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
    use crate::protocol::packets::packet_parts::RecipeBookEntry;

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
		assert_eq!(ser(&SlotDisplay::WithAnyPotion(Box::new(SlotDisplay::Empty))), vec![2, 0]);
		assert_eq!(ser(&SlotDisplay::OnlyWithComponent(Box::new(OnlyWithComponentSlotData { base: SlotDisplay::Empty, component_type: VarInt(9) }))), vec![3, 0, 9]);
		assert_eq!(ser(&SlotDisplay::Item(VarInt(7))), vec![4, 7]);
		assert_eq!(ser(&SlotDisplay::Tag(IDSet { typ: VarInt(0), tag_name: Some("a:b".parse().unwrap()), ids: None })), vec![6, 0, 3, b'a', b':', b'b']);
		assert_eq!(ser(&SlotDisplay::Dyed(Box::new(DyedSlotData { dye: SlotDisplay::Empty, target: SlotDisplay::AnyFuel }))), vec![7, 0, 1]);
		assert_eq!(ser(&SlotDisplay::Composite(Box::new(CompositeSlotData { options: PrefixedArray::new(vec![]) }))), vec![10, 0]);
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
	fn reads_recipe_book_add_body_from_26_3_server() {
		// Captured from a 26.3 server: the 2x2 planks -> crafting table recipe.
		// The wiki gives a different layout for `tag` and `item_stack`. If these drift again,
		// the client cannot finish the join, because every later field reads the wrong bytes.
		let planks_tag = |b: &mut Vec<u8>| {
			b.extend_from_slice(&[0x00, 0x10]);
			b.extend_from_slice(b"minecraft:planks");
		};
		let mut body = vec![0x01, 0xd3, 0x02, 0x01, 0x02, 0x02, 0x04];
		for _ in 0..4 {
			body.push(0x06);
			planks_tag(&mut body);
		}
		body.extend_from_slice(&[0x05, 0x95, 0x03, 0x01, 0x00, 0x00, 0x04, 0x95, 0x03, 0x00, 0x03, 0x01, 0x04]);
		for _ in 0..4 {
			planks_tag(&mut body);
		}
		body.extend_from_slice(&[0x02, 0x01]);

		let mut d = McDeserializer::new(&body);
		let entries = PrefixedArray::<RecipeBookEntry>::mc_deserialize(&mut d).unwrap();
		let replace = bool::mc_deserialize(&mut d).unwrap();
		assert!(d.is_at_end(), "all bytes must be used");
		assert!(replace);

		let entry = &entries.vec[0];
		let RecipeDisplay::CraftingShaped(shaped) = &entry.display else { panic!("expected shaped recipe, got {:?}", entry.display) };
		let planks = SlotDisplay::Tag(IDSet { typ: VarInt(0), tag_name: Some("minecraft:planks".parse().unwrap()), ids: None });
		assert_eq!(shaped.ingredients.vec, vec![planks.clone(), planks.clone(), planks.clone(), planks]);
		assert_eq!(shaped.result, SlotDisplay::ItemStack(ItemStackTemplate { item_id: VarInt(405), item_count: VarInt(1), components_to_add: vec![], components_to_remove: vec![] }));
		assert_eq!(shaped.crafting_station, SlotDisplay::Item(VarInt(405)));
		assert_eq!(entry.flags, 0x02);

		// Writing it back must give the same bytes, or a sandstone server sends a packet the client rejects.
		let mut s = McSerializer::new();
		entries.mc_serialize(&mut s).unwrap();
		replace.mc_serialize(&mut s).unwrap();
		assert_eq!(s.as_bytes(), &body[..]);
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
