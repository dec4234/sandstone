//! Defines a lot of random components of network packets. This is separate from packet.rs to reduce
//! clutter.

use sandstone_derive::McSerialize;
use uuid::Uuid;

use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;

// TODO: maybe we can make a derive tag for options? At the very least only the option section needs to
// be in a special body struct.
#[derive(McSerialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginPropertyElement {
	pub(crate) name: String,
	pub(crate) value: String,
	pub(crate) is_signed: bool,
	pub(crate) signature: Option<String>,
}

impl McDeserialize for LoginPropertyElement {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let name = String::mc_deserialize(deserializer)?;
		let value = String::mc_deserialize(deserializer)?;
		let is_signed = bool::mc_deserialize(deserializer)?;


		let signature = if is_signed {
			Some(String::mc_deserialize(deserializer)?)
		} else {
			None
		};

		Ok(Self {
			name,
			value,
			is_signed,
			signature,
		})
	}
}

#[derive(McSerialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginPluginSpec {
	pub(crate) message_id: VarInt,
	pub(crate) success: bool,
	pub(crate) data: Option<Vec<u8>>,
}

impl McDeserialize for LoginPluginSpec {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let message_id = VarInt::mc_deserialize(deserializer)?;
		let success = bool::mc_deserialize(deserializer)?;

		let data = if success {
			Some(Vec::<u8>::mc_deserialize(deserializer)?)
		} else {
			None
		};

		Ok(Self {
			message_id,
			success,
			data,
		})
	}
}

#[derive(McSerialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RemoveResourcePackSpec {
	pub(crate) has_uuid: bool,
	pub(crate) uuid: Option<Uuid>,
}

impl McDeserialize for RemoveResourcePackSpec {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let has_uuid = bool::mc_deserialize(deserializer)?;
		let uuid = if has_uuid {
			Some(Uuid::mc_deserialize(deserializer)?)
		} else {
			None
		};

		Ok(Self {
			has_uuid,
			uuid,
		})
	}
}

#[derive(McSerialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddResourcePackSpec {
	pub(crate) uuid: Uuid,
	pub(crate) url: String,
	pub(crate) hash: String,
	pub(crate) forced: bool,
	pub(crate) has_prompt_message: bool,
	pub(crate) prompt_message: Option<String>,
}

impl McDeserialize for AddResourcePackSpec {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let uuid = Uuid::mc_deserialize(deserializer)?;
		let url = String::mc_deserialize(deserializer)?;
		let hash = String::mc_deserialize(deserializer)?;
		let forced = bool::mc_deserialize(deserializer)?;
		let has_prompt_message = bool::mc_deserialize(deserializer)?;
		let prompt_message = if has_prompt_message {
			Some(String::mc_deserialize(deserializer)?)
		} else {
			None
		};

		Ok(Self {
			uuid,
			url,
			hash,
			forced,
			has_prompt_message,
			prompt_message,
		})
	}
}

#[derive(McSerialize, Debug, Clone, PartialEq, Eq)]
pub struct LoginCookieResponseSpec {
	key: String,
	has_payload: bool,
	payload_length: VarInt,
	payload: Option<Vec<u8>>,
}

impl McDeserialize for LoginCookieResponseSpec {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let key = String::mc_deserialize(deserializer)?;
		let has_payload = bool::mc_deserialize(deserializer)?;
		let payload_length = VarInt::mc_deserialize(deserializer)?;
		let payload = if has_payload {
			Some(Vec::<u8>::mc_deserialize(deserializer)?)
		} else {
			None
		};

		Ok(Self {
			key,
			has_payload,
			payload_length,
			payload,
		})
	}
}

#[derive(McSerialize, Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry {
	pub id: String,
	pub has_data: bool,
	pub data: Option<NbtCompound>,
}

impl McDeserialize for RegistryEntry {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let id = String::mc_deserialize(deserializer)?;
		let has_data = bool::mc_deserialize(deserializer)?;
		let data = if has_data {
			Some(NbtCompound::mc_deserialize(deserializer)?)
		} else {
			None
		};

		Ok(Self {
			id,
			has_data,
			data,
		})
	}
}