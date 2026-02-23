//! Defines a lot of random components of network packets. This is separate from packet.rs to reduce
//! clutter.

use crate::protocol::testing::McDefault;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};
use uuid::Uuid;

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::util::java::bitfield::BitField;

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginPluginSpec {
	pub(crate) message_id: VarInt,
	pub(crate) success: bool,
	#[mc(deserialize_if = success)]
	pub(crate) data: Option<Vec<u8>>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddResourcePackSpec {
	pub(crate) uuid: Uuid,
	pub(crate) url: String,
	pub(crate) hash: String,
	pub(crate) forced: bool,
	pub(crate) has_prompt_message: bool,
	#[mc(deserialize_if = has_prompt_message)]
	pub(crate) prompt_message: Option<String>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct LoginCookieResponseSpec {
	key: String,
	has_payload: bool,
	payload_length: VarInt,
	#[mc(deserialize_if = has_payload)]
	payload: Option<Vec<u8>>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct ResourcePackEntry {
	pub namespace: String,
	pub id: String,
	pub version: String
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct TagArray {
	pub identifier: String,
	pub payload: PrefixedArray<VarInt>
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct ProtocolPropertyElement {
	pub name: String,
	pub value: String,
	pub signature: PrefixedOptional<String>
}

#[derive(McDefault, Debug, Clone, PartialEq, Eq)]
pub struct PlayerAbilityFlags {
	pub invulnerable: bool,
	pub flying: bool,
	pub allow_flying: bool,
	pub creative_mode: bool
}

impl McSerialize for PlayerAbilityFlags {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let mut field = BitField::new(0u8);
		
		field.set_bit(0, self.invulnerable);
		field.set_bit(1, self.flying);
		field.set_bit(2, self.allow_flying);
		field.set_bit(3, self.creative_mode);
		
		field.mc_serialize(serializer)
	}
}

impl McDeserialize for PlayerAbilityFlags {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized
	{
		let field = BitField::<u8>::mc_deserialize(deserializer)?;
		
		Ok(Self {
			invulnerable: field.get_bit(0),
			flying: field.get_bit(1),
			allow_flying: field.get_bit(2),
			creative_mode: field.get_bit(3)
		})
	}
}