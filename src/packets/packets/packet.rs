use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::packets;
use crate::packets::packet_definer::{PacketState, PacketTrait, PacketVersionDefinition};
use crate::packets::packets::packet_component::LoginPropertyElement;
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::packets::serialization::serializer_handler::DeserializeResult;
use crate::packets::serialization::serializer_handler::StateBasedDeserializer;
use crate::protocol_details::datatypes::var_types::VarInt;

// https://wiki.vg/Protocol
packets!(V1_20 => {
	// HANDSHAKE
	Handshaking, HandshakingBody, 0x00, HANDSHAKING => {
		protocol_version: VarInt,
		server_address: String,
		port: u16,
		next_state: VarInt
	},
	
	// STATUS
	// Please note that the STATUS packets are defined elsewhere in status_packets.rs
	// They are provided here for completeness, however it is not reccomended to use these ones
	
	// Client-bound
	StatusResponse, StatusResponseBody, 0x00, STATUS => {
		response: String
	},

	PingResponse, PingResponseBody, 0x01, STATUS => {
        payload: u64
    },
	
	// Server-bound
	StatusRequest, StatusRequestBody, 0x00, STATUS => {
		// none
	},
	
	PingRequest, PingRequestBody, 0x01, STATUS => {
		payload: i64
	},
	
	// LOGIN
	
	// Client-bound
	Disconnect, DisconnectBody, 0x00, LOGIN => {
		reason: String
	},
	
	EncryptionRequest, EncryptionRequestBody, 0x01, LOGIN => {
		server_id: String,
		public_key_length: VarInt,
		public_key: Vec<u8>,
		verify_token_length: VarInt, // always 4 for Notchian servers
		verify_token: Vec<u8>
	},
	
	LoginSuccess, LoginSuccessBody, 0x02, LOGIN => {
		uuid: String,
		username: String,
		num_properties: VarInt,
		array: Vec<LoginPropertyElement>
	}
});

#[cfg(test)]
mod tests {
	use crate::packets::serialization::serializer_handler::McDeserializer;

	#[test]
	fn try_deserialize() {
		// vari       string                                            u16         vari
		// 251, 5,    9, 108, 111, 99, 97, 108, 104, 111, 115, 116,     99, 221,    1

		let vec: &[u8] = &[16, 0, 251, 5, 9, 108, 111, 99, 97, 108, 104, 111, 115, 116, 99, 221, 1];

		let mut deserializer = McDeserializer::new(vec);

		/*let p: raw_packet::PackagedPacket<v1_20> = raw_packet::PackagedPacket::mc_deserialize(&mut deserializer).unwrap();

		match p.data {
			v1_20::StatusRequest(_) => {}
			v1_20::Handshaking(b) => {println!("Address: {}", b.server_address)}
			v1_20::PingResponse(_) => {}
		}*/
	}
}