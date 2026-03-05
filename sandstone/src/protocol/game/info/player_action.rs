use crate::protocol::game::info::player::PlayerInfoUpdateType;
use crate::protocol::packets::packet_component::ProtocolPropertyElement;
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
#[derive(Debug, Clone, PartialEq)]
pub struct EnumSet {
	field: BitField<u8>,
}

impl EnumSet {
	pub fn new() -> Self {
		Self { field: BitField::new(0) }
	}

	pub fn from_raw(value: u8) -> Self {
		Self { field: BitField::new(value) }
	}

	pub fn has(&self, action: &PlayerInfoUpdateType) -> bool {
		self.field.get_bit(action.get_bit_index())
	}

	pub fn set(&mut self, action: &PlayerInfoUpdateType, value: bool) {
		self.field.set_bit(action.get_bit_index(), value);
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		Ok(Self { field: BitField::mc_deserialize(deserializer)? })
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
		if mask & 0x01 != 0 && let Some(data) = &self.add_player {
			data.mc_serialize(serializer)?;
		}
		if mask & 0x02 != 0 && let Some(data) = &self.initialize_chat {
			data.mc_serialize(serializer)?;
		}
		if mask & 0x04 != 0 && let Some(data) = &self.update_game_mode {
			data.mc_serialize(serializer)?;
		}
		if mask & 0x08 != 0 && let Some(data) = &self.update_listed {
			data.mc_serialize(serializer)?;
		}
		if mask & 0x10 != 0 && let Some(data) = &self.update_latency {
			data.mc_serialize(serializer)?;
		}
		if mask & 0x20 != 0 && let Some(data) = &self.update_display_name {
			data.mc_serialize(serializer)?;
		}
		if mask & 0x40 != 0 && let Some(data) = &self.update_list_priority {
			data.mc_serialize(serializer)?;
		}
		if mask & 0x80 != 0 && let Some(data) = &self.update_hat {
			data.mc_serialize(serializer)?;
		}
		Ok(())
	}

	pub fn deserialize_with_mask<'a>(mask: u8, deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let add_player = if mask & 0x01 != 0 {
			Some(AddPlayerData::mc_deserialize(deserializer)?)
		} else { None };

		let initialize_chat = if mask & 0x02 != 0 {
			Some(PrefixedOptional::<ChatSessionData>::mc_deserialize(deserializer)?)
		} else { None };

		let update_game_mode = if mask & 0x04 != 0 {
			Some(VarInt::mc_deserialize(deserializer)?)
		} else { None };

		let update_listed = if mask & 0x08 != 0 {
			Some(bool::mc_deserialize(deserializer)?)
		} else { None };

		let update_latency = if mask & 0x10 != 0 {
			Some(VarInt::mc_deserialize(deserializer)?)
		} else { None };

		let update_display_name = if mask & 0x20 != 0 {
			Some(PrefixedOptional::<TextComponent>::mc_deserialize(deserializer)?)
		} else { None };

		let update_list_priority = if mask & 0x40 != 0 {
			Some(VarInt::mc_deserialize(deserializer)?)
		} else { None };

		let update_hat = if mask & 0x80 != 0 {
			Some(bool::mc_deserialize(deserializer)?)
		} else { None };

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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let actions = EnumSet::mc_deserialize(deserializer)?;
		let mask = actions.raw();
		let count = VarInt::mc_deserialize(deserializer)?.0;
		let mut entries = Vec::with_capacity(count as usize);
		for _ in 0..count {
			let uuid = Uuid::mc_deserialize(deserializer)?;
			let player_actions = PlayerAction::deserialize_with_mask(mask, deserializer)?;
			entries.push(PlayerInfoEntry { uuid, actions: player_actions });
		}
		Ok(Self { actions, entries })
	}
}

impl McDefault for PlayerInfoUpdateData {
	fn mc_default() -> Self {
		Self { actions: EnumSet::new(), entries: vec![] }
	}
}
