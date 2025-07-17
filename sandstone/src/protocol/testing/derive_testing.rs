//! Tests the functionality of the 'McDeserialize' and 'McSerialize' derive macros.

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use sandstone_derive::{McDeserialize, McSerialize};

#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestPacket {
	pub field1: bool,
	#[mc(deserialize_if = field1)]
	pub field2: Option<String>
}

#[cfg(test)]
mod tests {
	use crate::protocol::serialization::{McDeserialize, McSerialize};

	#[test]
	fn test_serialize_deserialize() {
		let packet = super::TestPacket { field1: true, field2: Some("hello".to_string()) };
		let mut serializer = super::McSerializer::init_size(100);
		packet.mc_serialize(&mut serializer).unwrap();
		
		let mut deserializer = super::McDeserializer::new(&serializer.output);
		let deserialized_packet = super::TestPacket::mc_deserialize(&mut deserializer).unwrap();
		
		assert_eq!(packet, deserialized_packet);
	}

	#[test]
	fn test_dont_deserialize() {
		let packet = super::TestPacket { field1: false, field2: Some("hello".to_string()) };
		let mut serializer = super::McSerializer::init_size(100);
		packet.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = super::McDeserializer::new(&serializer.output);
		let deserialized_packet = super::TestPacket::mc_deserialize(&mut deserializer).unwrap();

		let actual = super::TestPacket { field1: false, field2: None };
		assert_eq!(deserialized_packet, actual);

		let packet = super::TestPacket { field1: true, field2: Some("hello".to_string()) };
		let mut serializer = super::McSerializer::init_size(100);
		packet.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = super::McDeserializer::new(&serializer.output);
		let deserialized_packet = super::TestPacket::mc_deserialize(&mut deserializer).unwrap();

		assert_eq!(deserialized_packet, packet);
	}
}