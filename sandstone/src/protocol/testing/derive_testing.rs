//! Tests the functionality of the 'McDeserialize' and 'McSerialize' derive macros.

use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDeserialize, McSerialize};

#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestPacket {
	pub field1: bool,
	#[mc(deserialize_if = field1)]
	pub field2: Option<String>,
}

/// Exercises the enum support of the derives: a leading VarInt discriminant selects the variant,
/// then that variant's body is (de)serialized. Covers named, unit, and unnamed variants.
#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum TestEnum {
	Named { a: u8, b: String } = 0,
	Unit = 1,
	Unnamed(u32) = 2,
}

#[cfg(test)]
mod tests {
	use crate::protocol::serialization::{McDeserialize, McSerialize};

	#[test]
	fn test_serialize_deserialize() {
		let packet = super::TestPacket {
			field1: true,
			field2: Some("hello".to_string()),
		};
		let mut serializer = super::McSerializer::init_size(100);
		packet.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = super::McDeserializer::new(&serializer.output);
		let deserialized_packet = super::TestPacket::mc_deserialize(&mut deserializer).unwrap();

		assert_eq!(packet, deserialized_packet);
	}

	#[test]
	fn test_dont_deserialize() {
		let packet = super::TestPacket {
			field1: false,
			field2: Some("hello".to_string()),
		};
		let mut serializer = super::McSerializer::init_size(100);
		packet.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = super::McDeserializer::new(&serializer.output);
		let deserialized_packet = super::TestPacket::mc_deserialize(&mut deserializer).unwrap();

		let actual = super::TestPacket {
			field1: false,
			field2: None,
		};
		assert_eq!(deserialized_packet, actual);

		let packet = super::TestPacket {
			field1: true,
			field2: Some("hello".to_string()),
		};
		let mut serializer = super::McSerializer::init_size(100);
		packet.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = super::McDeserializer::new(&serializer.output);
		let deserialized_packet = super::TestPacket::mc_deserialize(&mut deserializer).unwrap();

		assert_eq!(deserialized_packet, packet);
	}

	#[test]
	fn test_enum_round_trips_all_variant_shapes() {
		for variant in [
			super::TestEnum::Named {
				a: 7,
				b: "hi".to_string(),
			},
			super::TestEnum::Unit,
			super::TestEnum::Unnamed(0xDEAD_BEEF),
		] {
			let mut serializer = super::McSerializer::init_size(100);
			variant.mc_serialize(&mut serializer).unwrap();

			let mut deserializer = super::McDeserializer::new(&serializer.output);
			let decoded = super::TestEnum::mc_deserialize(&mut deserializer).unwrap();

			assert_eq!(variant, decoded);
		}
	}

	#[test]
	fn test_enum_writes_discriminant_first() {
		// The discriminant must be written as a leading VarInt so the reader can pick the variant.
		// `Unit` (= 1) carries no body, so the entire wire form is just the VarInt `1`.
		let mut serializer = super::McSerializer::init_size(100);
		super::TestEnum::Unit.mc_serialize(&mut serializer).unwrap();
		assert_eq!(serializer.output.as_slice(), &[1u8]);
	}
}
