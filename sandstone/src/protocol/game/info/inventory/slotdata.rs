use crate::protocol::game::info::inventory::components::StructuredComponent;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;

// https://minecraft.wiki/w/Java_Edition_protocol/Slot_data
#[derive(Debug, Clone, PartialEq)]
pub struct SlotData {
	pub item_count: VarInt,
	pub item_id: Option<VarInt>,
	pub components_to_add: Vec<StructuredComponent>,
	pub components_to_remove: Vec<VarInt>,
}

impl McDefault for SlotData {
	fn mc_default() -> Self {
		Self {
			item_count: VarInt(1),
			item_id: Some(VarInt(1)),
			components_to_add: vec![],
			components_to_remove: vec![],
		}
	}
}

impl McSerialize for SlotData {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.item_count.mc_serialize(serializer)?;
		if self.item_count.0 > 0 {
			if let Some(id) = &self.item_id {
				id.mc_serialize(serializer)?;
			}
			VarInt(self.components_to_add.len() as i32).mc_serialize(serializer)?;
			VarInt(self.components_to_remove.len() as i32).mc_serialize(serializer)?;
			for component in &self.components_to_add {
				component.mc_serialize(serializer)?;
			}
			for id in &self.components_to_remove {
				id.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for SlotData {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let item_count = VarInt::mc_deserialize(deserializer)?;
		if item_count.0 == 0 {
			return Ok(Self {
				item_count,
				item_id: None,
				components_to_add: vec![],
				components_to_remove: vec![],
			});
		}
		let item_id = VarInt::mc_deserialize(deserializer)?;
		let num_add = VarInt::mc_deserialize(deserializer)?;
		let num_remove = VarInt::mc_deserialize(deserializer)?;

		let mut components_to_add = Vec::with_capacity(num_add.0 as usize);
		for _ in 0..num_add.0 {
			components_to_add.push(StructuredComponent::mc_deserialize(deserializer)?);
		}

		let mut components_to_remove = Vec::with_capacity(num_remove.0 as usize);
		for _ in 0..num_remove.0 {
			components_to_remove.push(VarInt::mc_deserialize(deserializer)?);
		}

		Ok(Self {
			item_count,
			item_id: Some(item_id),
			components_to_add,
			components_to_remove,
		})
	}
}
