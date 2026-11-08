use crate::bitflag;
use crate::protocol::game::player::inventory::slotdata::SlotData;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
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

	fn display(flags: AdvancementFlags, background: Option<&str>) -> AdvancementDisplay {
		AdvancementDisplay {
			title: TextComponent::new("Title"),
			description: TextComponent::new("Desc"),
			icon: SlotData { item_count: VarInt(0), item_id: None, components_to_add: vec![], components_to_remove: vec![] },
			frame_type: AdvancementFrameType::Challenge,
			flags,
			background_texture: background.map(|s| s.to_string()),
			x: 1.0,
			y: 2.0,
		}
	}

	#[test]
	fn flag_bits_match_wiki() {
		// Wiki (Advancement Display): an Int with 0x01 has background, 0x02 show toast, 0x04 hidden.
		assert_eq!(ser(&AdvancementFlags::new(true, false, false)), vec![0, 0, 0, 0x01]);
		assert_eq!(ser(&AdvancementFlags::new(false, true, false)), vec![0, 0, 0, 0x02]);
		assert_eq!(ser(&AdvancementFlags::new(false, false, true)), vec![0, 0, 0, 0x04]);
	}

	#[test]
	fn background_texture_only_sent_when_flag_set() {
		// The client reads the texture only when bit 0x01 is set. X and Y follow it, so a wrong
		// flag shifts every later field.
		let without = display(AdvancementFlags::new(false, true, false), None);
		let with = display(AdvancementFlags::new(true, true, false), Some("minecraft:bg"));
		let extra = ser(&with).len() - ser(&without).len();
		assert_eq!(extra, 1 + "minecraft:bg".len());
		round_trip(without);
		round_trip(with);
	}

	#[test]
	fn advancement_round_trips() {
		round_trip(Advancement {
			parent_id: PrefixedOptional::new(None),
			display_data: PrefixedOptional::new(None),
			nested_requirements: PrefixedArray::new(vec![]),
			sends_telemetry_data: false,
		});
		round_trip(Advancement {
			parent_id: PrefixedOptional::new(Some("minecraft:story/root".into())),
			display_data: PrefixedOptional::new(Some(display(AdvancementFlags::new(true, true, true), Some("minecraft:bg")))),
			nested_requirements: PrefixedArray::new(vec![PrefixedArray::new(vec!["a".into(), "b".into()]), PrefixedArray::new(vec![])]),
			sends_telemetry_data: true,
		});
	}

	#[test]
	fn frame_type_ids_match_wiki() {
		assert_eq!(ser(&AdvancementFrameType::Task), vec![0]);
		assert_eq!(ser(&AdvancementFrameType::Goal), vec![2]);
	}
}
