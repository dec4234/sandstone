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
