use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginPropertyElement {
	name: String,
	value: String,
	is_signed: bool,
	signature: Option<String>
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
