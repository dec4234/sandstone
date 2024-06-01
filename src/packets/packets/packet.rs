use uuid::Uuid;

use crate::packets;
use crate::packets::packet_definer::{PacketDirection, PacketState};
use crate::packets::packets::packet_component::{AddResourcePackSpec, LoginPluginSpec, RemoveResourcePackSpec};
use crate::packets::packets::packet_component::LoginPropertyElement;
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::packets::serialization::serializer_handler::DeserializeResult;
use crate::packets::serialization::serializer_handler::StateBasedDeserializer;
use crate::packets::status::status_packets::StatusResponseSpec;
use crate::protocol_details::datatypes::chat::TextComponent;
use crate::protocol_details::datatypes::nbt::nbt::NbtCompound;
use crate::protocol_details::datatypes::var_types::VarInt;

/*
This file defines the packets for the most recent supported version of the Protocol

It has a couple of key responsibilities:
- Define the packets that are used in the protocol
- Define the serialization and deserialization for each packet
- Provide vital information about each packet such as the packet ID, the packet state, and the packet direction
*/

// https://wiki.vg/Protocol
packets!(V1_20 => {
	// HANDSHAKE
	Handshaking, HandshakingBody, 0x00, HANDSHAKING, SERVER => {
		protocol_version: VarInt,
		server_address: String,
		port: u16,
		next_state: VarInt
	},
	
	// STATUS
	// Please note that the STATUS packets are defined elsewhere in status_packets.rs
	// They are provided here for completeness, however it is not reccomended to use these ones
	
	// Client-bound
	StatusResponse, StatusResponseBody, 0x00, STATUS, CLIENT => {
		response: StatusResponseSpec
	},

	PingResponse, PingResponseBody, 0x01, STATUS, CLIENT => {
        payload: u64
    },
	
	// Server-bound
	StatusRequest, StatusRequestBody, 0x00, STATUS, SERVER => {
		// none
	},
	
	PingRequest, PingRequestBody, 0x01, STATUS, SERVER => {
		payload: i64
	},
	
	// LOGIN
	
	// Client-bound
	Disconnect, DisconnectBody, 0x00, LOGIN, CLIENT => {
		reason: TextComponent
	},
	
	EncryptionRequest, EncryptionRequestBody, 0x01, LOGIN, CLIENT => {
		server_id: String,
		public_key_length: VarInt,
		public_key: Vec<u8>,
		verify_token_length: VarInt, // always 4 for Notchian servers
		verify_token: Vec<u8>
	},
	
	LoginSuccess, LoginSuccessBody, 0x02, LOGIN, CLIENT => {
		uuid: String,
		username: String,
		num_properties: VarInt,
		array: Vec<LoginPropertyElement>
	},
	
	SetCompression, SetCompressionBody, 0x03, LOGIN, CLIENT => {
		threshold: VarInt
	},
	
	LoginPluginRequest, LoginPluginRequestBody, 0x04, LOGIN, CLIENT => {
		message_id: VarInt,
		channel: String,
		data: Vec<u8>
	},
	
	// Server-bound
	LoginStart, LoginStartBody, 0x00, LOGIN, SERVER => {
		username: String,
		uuid: Uuid
	},
	
	EncryptionResponse, EncryptionResponseBody, 0x01, LOGIN, SERVER => {
		shared_secret_length: VarInt,
		shared_secret: Vec<u8>,
		verify_token_length: VarInt,
		verify_token: Vec<u8>
	},
	
	LoginPluginResponse, LoginPluginResponseBody, 0x02, LOGIN, SERVER => {
		response: LoginPluginSpec
	},
	
	LoginAcknowledged, LoginAcknowledgedBody, 0x03, LOGIN, SERVER => {
		// none
	},
	
	// CONFIGURATION
	
	// Client-bound
	PluginMessage, PluginMessageBody, 0x00, LOGIN, CLIENT => {
		channel: String,
		data: Vec<u8>
	},

	ConfigDisconnect, ConfigDisconnectBody, 0x01, LOGIN, CLIENT => {
		reason: TextComponent
	},

	FinishConfiguration, FinishConfigurationBody, 0x02, LOGIN, CLIENT => {
		// none
	},

	KeepAlive, KeepAliveBody, 0x03, LOGIN, CLIENT => {
		keep_alive_id: i64
	},

	ConfigurationPing, ConfigurationPingBody, 0x04, LOGIN, CLIENT => {
		payload: i32
	},

	RegistryData, RegistryDataBody, 0x05, LOGIN, CLIENT => {
		registry_codec: NbtCompound
	},

	RemoveResourcePack, RemoveResourcePackBody, 0x06, LOGIN, CLIENT => {
		spec: RemoveResourcePackSpec
	},

	AddResourcePack, AddResourcePackBody, 0x07, LOGIN, CLIENT => {
		spec: AddResourcePackSpec
	},

	FeatureFlags, FeatureFlagsBody, 0x08, LOGIN, CLIENT => {
		total: VarInt,
		flags: Vec<String>
	},


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