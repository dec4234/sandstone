use sandstone_derive::{McDeserialize, McSerialize};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::serialization::serializer_error::SerializingErr;

#[derive(McSerialize, McDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestPacket {
	pub field1: bool,
}