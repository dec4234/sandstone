use std::fmt::{Debug, Display};

use anyhow::Result;

use crate::packets::packet_definer::PacketState;
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol_details::datatypes::var_types::VarInt;

#[derive(Debug, Clone)]
pub struct PackagedPacket<P: McSerialize + StateBasedDeserializer> {
	length: VarInt,
	packet_id: VarInt,
	pub data: P
}

impl<P: McSerialize + StateBasedDeserializer> McSerialize for PackagedPacket<P> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		self.length.mc_serialize(serializer)?;
		self.packet_id.mc_serialize(serializer)?;
		self.data.mc_serialize(serializer)?;

		Ok(())
	}
}

impl<P: McSerialize + StateBasedDeserializer> StateBasedDeserializer for PackagedPacket<P> {
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: &PacketState) -> DeserializeResult<'a, Self> where Self: Sized {
		let raw = PackagedPacket {
			length: VarInt::mc_deserialize(deserializer)?,
			packet_id: VarInt::mc_deserialize(deserializer)?,
			data: P::deserialize_state(deserializer, state)?,
		};
		
		return Ok(raw);
	}
}