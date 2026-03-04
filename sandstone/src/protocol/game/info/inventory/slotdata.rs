use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;

// https://minecraft.wiki/w/Java_Edition_protocol/Slot_data
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct SlotData {
	pub item_count: VarInt,
	pub item_id: Option<VarInt>,
	pub num_components_to_add: Option<VarInt>,
	pub num_components_to_remove: Option<VarInt>,
	// todo: component data arrays
}

impl McDefault for SlotData {
	fn mc_default() -> Self {
		Self {
			item_count: VarInt(1),
			item_id: Some(VarInt(1)),
			num_components_to_add: Some(VarInt(0)),
			num_components_to_remove: Some(VarInt(0)),
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
			if let Some(add) = &self.num_components_to_add {
				add.mc_serialize(serializer)?;
			}
			if let Some(remove) = &self.num_components_to_remove {
				remove.mc_serialize(serializer)?;
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
				num_components_to_add: None,
				num_components_to_remove: None,
			});
		}
		let item_id = VarInt::mc_deserialize(deserializer)?;
		let num_add = VarInt::mc_deserialize(deserializer)?;
		let num_remove = VarInt::mc_deserialize(deserializer)?;

		for _ in 0..num_remove.0 {
			VarInt::mc_deserialize(deserializer)?;
		}

		if num_add.0 > 0 {
			return Err(SerializingErr::DeserializationError(
				format!("SlotData with {} components to add is not yet supported", num_add.0)
			));
		}

		Ok(Self {
			item_count,
			item_id: Some(item_id),
			num_components_to_add: Some(num_add),
			num_components_to_remove: Some(num_remove),
		})
	}
}