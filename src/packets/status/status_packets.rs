use crate::packets::packet_definer::{PacketState, PacketTrait};
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol_details::datatypes::var_types::VarInt;

pub struct UniversalHandshakePacket {
	pub protocol_version: VarInt,
	pub server_address: String,
	pub server_port: u16,
	pub next_state: VarInt
}

impl UniversalHandshakePacket {
	pub fn new(protocol_version: VarInt, server_address: String, server_port: u16, next_state: VarInt) -> Self {
		Self {
			protocol_version,
			server_address,
			server_port,
			next_state
		}
	}
}

impl McSerialize for UniversalHandshakePacket {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		self.protocol_version.mc_serialize(serializer)?;
		self.server_address.mc_serialize(serializer)?;
		self.server_port.mc_serialize(serializer)?;
		self.next_state.mc_serialize(serializer)?;

		Ok(())
	}
}

impl StateBasedDeserializer for UniversalHandshakePacket {
	fn deserialize_state<'a>(deserializer: &'a mut McDeserializer, state: &PacketState) -> DeserializeResult<'a, Self> {
		if state != &PacketState::HANDSHAKING {
			return Err(SerializingErr::InvalidPacketState);
		}
		
		let raw = UniversalHandshakePacket {
			protocol_version: VarInt::mc_deserialize(deserializer)?,
			server_address: String::mc_deserialize(deserializer)?,
			server_port: u16::mc_deserialize(deserializer)?,
			next_state: VarInt::mc_deserialize(deserializer)?,
		};
		
		Ok(raw)
	}
}

impl PacketTrait for UniversalHandshakePacket {
	fn packet_id() -> u8 {
		0x00
	}

	fn state() -> PacketState {
		PacketState::HANDSHAKING
	}
}

