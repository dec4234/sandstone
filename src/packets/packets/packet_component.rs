use uuid::Uuid;

use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol_details::datatypes::var_types::VarInt;

/*
Defines a lot of random components of network packets. This is separate from packet.rs to reduce
clutter.
 */

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginPropertyElement {
	pub(crate) name: String,
	pub(crate) value: String,
	pub(crate) is_signed: bool,
	pub(crate) signature: Option<String>
}

// https://docs.rs/syn/0.15.18/syn/#example-of-a-custom-derive
impl McSerialize for LoginPropertyElement { // TODO: possibly create derive macros for McSerialize / McDeserialize
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		self.name.mc_serialize(serializer)?;
		self.value.mc_serialize(serializer)?;
		self.is_signed.mc_serialize(serializer)?;
		self.signature.mc_serialize(serializer)?;
		
		Ok(())
	}
}

impl McDeserialize for LoginPropertyElement {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
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
			signature
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginPluginSpec {
	pub(crate) message_id: VarInt,
	pub(crate) success: bool,
	pub(crate) data: Option<Vec<u8>>
}

impl McSerialize for LoginPluginSpec {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		self.message_id.mc_serialize(serializer)?;
		self.success.mc_serialize(serializer)?;
		self.data.mc_serialize(serializer)?;
		
		Ok(())
	}
}

impl McDeserialize for LoginPluginSpec {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
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
			data
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RemoveResourcePackSpec {
	pub(crate) has_uuid: bool,
	pub(crate) uuid: Option<Uuid>
}

impl McSerialize for RemoveResourcePackSpec {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		self.has_uuid.mc_serialize(serializer)?;
		self.uuid.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for RemoveResourcePackSpec {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
		let has_uuid = bool::mc_deserialize(deserializer)?;
		let uuid = if has_uuid {
			Some(Uuid::mc_deserialize(deserializer)?)
		} else {
			None
		};

		Ok(Self {
			has_uuid,
			uuid
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddResourcePackSpec {
	pub(crate) uuid: Uuid,
	pub(crate) url: String,
	pub(crate) hash: String,
	pub(crate) forced: bool,
	pub(crate) has_prompt_message: bool,
	pub(crate) prompt_message: Option<String>
}

impl McSerialize for AddResourcePackSpec {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		self.uuid.mc_serialize(serializer)?;
		self.url.mc_serialize(serializer)?;
		self.hash.mc_serialize(serializer)?;
		self.forced.mc_serialize(serializer)?;
		self.has_prompt_message.mc_serialize(serializer)?;
		self.prompt_message.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for AddResourcePackSpec {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> {
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
			prompt_message
		})
	}
}