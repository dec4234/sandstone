use std::fmt::{Debug, Display};

use anyhow::Result;

use crate::packets::packet_definer::PacketState;
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol_details::datatypes::var_types::VarInt;

#[derive(Debug, Clone)]
pub struct PackagedPacket<P: McSerialize + StateBasedDeserializer> {
	packet_id: VarInt,
	pub data: P
}

impl<P: McSerialize + StateBasedDeserializer> PackagedPacket<P> {
	pub fn new(packet_id: VarInt, data: P) -> Self {
		Self {
			packet_id,
			data
		}
	}
}

impl<P: McSerialize + StateBasedDeserializer> McSerialize for PackagedPacket<P> {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		let mut length_serializer = McSerializer::new();
		self.data.mc_serialize(&mut length_serializer)?;
		
		VarInt(length_serializer.output.len() as i32 + self.packet_id.to_bytes().len() as i32).mc_serialize(serializer)?;
		self.packet_id.mc_serialize(serializer)?;
		serializer.merge(length_serializer);
		

		Ok(())
	}
}

impl<P: McSerialize + StateBasedDeserializer> StateBasedDeserializer for PackagedPacket<P> {
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: &PacketState) -> DeserializeResult<'a, Self> where Self: Sized {
		let length = VarInt::mc_deserialize(deserializer)?;
		let packet_id = VarInt::mc_deserialize(deserializer)?;
		let data = P::deserialize_state(&mut deserializer.sub_deserializer_length(length.0 as usize - packet_id.to_bytes().len())
				.map_err(|e| SerializingErr::UniqueFailure(e.to_string()))?, state)?;
		
		let raw = PackagedPacket {
			packet_id,
			data
		};
		
		return Ok(raw);
	}
}