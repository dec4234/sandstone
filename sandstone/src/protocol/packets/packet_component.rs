//! Defines a lot of random components of network packets. This is separate from packet.rs to reduce
//! clutter.

use crate::protocol::testing::McDefault;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};
use uuid::Uuid;

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol_types::datatypes::var_types::VarInt;

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