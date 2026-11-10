use crate::protocol::packets::packet_parts::ProtocolPropertyElement;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::util::java::bitfield::BitField;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};
use uuid::Uuid;

/// Only used for Player Info Updates.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct EnumSet {
	field: BitField<u8>,
}

impl EnumSet {
	pub fn new() -> Self {
		Self {
			field: BitField::new(0),
		}
	}

	pub fn from_raw(value: u8) -> Self {
		Self {
			field: BitField::new(value),
		}
	}

	pub fn raw(&self) -> u8 {
		let mut val = 0u8;
		for i in 0..8 {
			if self.field.get_bit(i) {
				val |= 1 << i;
			}
		}
		val
	}
}

impl McSerialize for EnumSet {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.field.mc_serialize(serializer)
	}
}

impl McDeserialize for EnumSet {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		Ok(Self {
			field: BitField::mc_deserialize(deserializer)?,
		})
	}
}

impl McDefault for EnumSet {
	fn mc_default() -> Self {
		Self::new()
	}
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct AddPlayerData {
	pub name: String,
	pub properties: PrefixedArray<ProtocolPropertyElement>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct ChatSessionData {
	pub session_id: Uuid,
	pub key_expiry: i64,
	pub public_key: PrefixedArray<u8>,
	pub key_signature: PrefixedArray<u8>,
}

/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#player-info:player-actions
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerAction {
	pub add_player: Option<AddPlayerData>,
	pub initialize_chat: Option<PrefixedOptional<ChatSessionData>>,
	pub update_game_mode: Option<VarInt>,
	pub update_listed: Option<bool>,
	pub update_latency: Option<VarInt>,
	pub update_display_name: Option<PrefixedOptional<TextComponent>>,
	pub update_list_priority: Option<VarInt>,
	pub update_hat: Option<bool>,
}

impl PlayerAction {
	pub fn serialize_with_mask(&self, mask: u8, serializer: &mut McSerializer) -> SerializingResult<()> {
		if mask & 0x01 != 0
			&& let Some(data) = &self.add_player
		{
			data.mc_serialize(serializer)?;
		}
		if mask & 0x02 != 0
			&& let Some(data) = &self.initialize_chat
		{
			data.mc_serialize(serializer)?;
		}
		if mask & 0x04 != 0
			&& let Some(data) = &self.update_game_mode
		{
			data.mc_serialize(serializer)?;
		}
		if mask & 0x08 != 0
			&& let Some(data) = &self.update_listed
		{
			data.mc_serialize(serializer)?;
		}
		if mask & 0x10 != 0
			&& let Some(data) = &self.update_latency
		{
			data.mc_serialize(serializer)?;
		}
		if mask & 0x20 != 0
			&& let Some(data) = &self.update_display_name
		{
			data.mc_serialize(serializer)?;
		}
		if mask & 0x40 != 0
			&& let Some(data) = &self.update_list_priority
		{
			data.mc_serialize(serializer)?;
		}
		if mask & 0x80 != 0
			&& let Some(data) = &self.update_hat
		{
			data.mc_serialize(serializer)?;
		}
		Ok(())
	}

	pub fn deserialize_with_mask<'a>(mask: u8, deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let add_player = if mask & 0x01 != 0 { Some(AddPlayerData::mc_deserialize(deserializer)?) } else { None };

		let initialize_chat = if mask & 0x02 != 0 {
			Some(PrefixedOptional::<ChatSessionData>::mc_deserialize(deserializer)?)
		} else {
			None
		};

		let update_game_mode = if mask & 0x04 != 0 { Some(VarInt::mc_deserialize(deserializer)?) } else { None };

		let update_listed = if mask & 0x08 != 0 { Some(bool::mc_deserialize(deserializer)?) } else { None };

		let update_latency = if mask & 0x10 != 0 { Some(VarInt::mc_deserialize(deserializer)?) } else { None };

		let update_display_name = if mask & 0x20 != 0 {
			Some(PrefixedOptional::<TextComponent>::mc_deserialize(deserializer)?)
		} else {
			None
		};

		let update_list_priority = if mask & 0x40 != 0 { Some(VarInt::mc_deserialize(deserializer)?) } else { None };

		let update_hat = if mask & 0x80 != 0 { Some(bool::mc_deserialize(deserializer)?) } else { None };

		Ok(Self {
			add_player,
			initialize_chat,
			update_game_mode,
			update_listed,
			update_latency,
			update_display_name,
			update_list_priority,
			update_hat,
		})
	}
}

impl McDefault for PlayerAction {
	fn mc_default() -> Self {
		Self {
			add_player: None,
			initialize_chat: None,
			update_game_mode: None,
			update_listed: None,
			update_latency: None,
			update_display_name: None,
			update_list_priority: None,
			update_hat: None,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerInfoEntry {
	pub uuid: Uuid,
	pub actions: PlayerAction,
}

// https://minecraft.wiki/w/Java_Edition_protocol/Packets#player-info:player-actions
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerInfoUpdateData {
	pub actions: EnumSet,
	pub entries: Vec<PlayerInfoEntry>,
}

impl McSerialize for PlayerInfoUpdateData {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		self.actions.mc_serialize(serializer)?;
		VarInt(self.entries.len() as i32).mc_serialize(serializer)?;
		let mask = self.actions.raw();
		for entry in &self.entries {
			entry.uuid.mc_serialize(serializer)?;
			entry.actions.serialize_with_mask(mask, serializer)?;
		}
		Ok(())
	}
}

impl McDeserialize for PlayerInfoUpdateData {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let actions = EnumSet::mc_deserialize(deserializer)?;
		let mask = actions.raw();
		let count = VarInt::mc_deserialize(deserializer)?.0;
		let mut entries = Vec::with_capacity(deserializer.prealloc_capacity(count)?);
		for _ in 0..count {
			let uuid = Uuid::mc_deserialize(deserializer)?;
			let player_actions = PlayerAction::deserialize_with_mask(mask, deserializer)?;
			entries.push(PlayerInfoEntry {
				uuid,
				actions: player_actions,
			});
		}
		Ok(Self {
			actions,
			entries,
		})
	}
}

impl McDefault for PlayerInfoUpdateData {
	fn mc_default() -> Self {
		Self {
			actions: EnumSet::new(),
			entries: vec![],
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	fn ser<T: McSerialize>(v: &T) -> Vec<u8> {
		let mut s = McSerializer::new();
		v.mc_serialize(&mut s).unwrap();
		s.as_bytes().to_vec()
	}

	fn empty_action() -> PlayerAction {
		PlayerAction::mc_default()
	}

	#[test]
	fn enum_set_raw_keeps_every_bit() {
		// The mask tells the client which action fields follow. One lost bit shifts every field after it.
		for raw in [0x00u8, 0x01, 0x09, 0xA5, 0xFF] {
			assert_eq!(EnumSet::from_raw(raw).raw(), raw);
			assert_eq!(ser(&EnumSet::from_raw(raw)), vec![raw]);
		}
	}

	#[test]
	fn mask_controls_which_fields_are_written() {
		// Fields that are Some but not in the mask must not go on the wire.
		let mut action = empty_action();
		action.update_listed = Some(true);
		action.update_latency = Some(VarInt(50));
		let mut s = McSerializer::new();
		action.serialize_with_mask(0x08, &mut s).unwrap();
		assert_eq!(s.as_bytes(), &[1]);
	}

	#[test]
	fn fields_are_written_in_bit_order() {
		// Wire order follows the bit order, low bit first.
		let mut action = empty_action();
		action.update_game_mode = Some(VarInt(3)); // 0x04
		action.update_latency = Some(VarInt(7)); // 0x10
		action.update_hat = Some(false); // 0x80
		let mut s = McSerializer::new();
		action.serialize_with_mask(0x94, &mut s).unwrap();
		assert_eq!(s.as_bytes(), &[3, 7, 0]);
	}

	#[test]
	fn player_info_update_round_trips() {
		let mut action = empty_action();
		action.add_player = Some(AddPlayerData { name: "Steve".into(), properties: PrefixedArray::new(vec![]) });
		action.update_listed = Some(true);
		action.update_list_priority = Some(VarInt(2));
		let data = PlayerInfoUpdateData {
			actions: EnumSet::from_raw(0x01 | 0x08 | 0x40),
			entries: vec![PlayerInfoEntry { uuid: Uuid::from_u128(1), actions: action }],
		};
		let bytes = ser(&data);
		// mask, entry count, then the 16 byte UUID.
		assert_eq!(&bytes[..2], &[0x49, 1]);
		assert_eq!(&bytes[2..18], Uuid::from_u128(1).as_bytes());
		let mut d = McDeserializer::new(&bytes);
		assert_eq!(PlayerInfoUpdateData::mc_deserialize(&mut d).unwrap(), data);
		assert!(d.is_at_end());
	}

	#[test]
	fn empty_mask_sends_only_uuids() {
		let data = PlayerInfoUpdateData {
			actions: EnumSet::new(),
			entries: vec![PlayerInfoEntry { uuid: Uuid::nil(), actions: empty_action() }],
		};
		assert_eq!(ser(&data).len(), 1 + 1 + 16);
	}
}
