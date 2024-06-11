use uuid::Uuid;

use crate::packets;
use crate::protocol::packet_definer::{PacketDirection, PacketState};
use crate::protocol::packets::packet_component::{AddResourcePackSpec, LoginPluginSpec, RemoveResourcePackSpec};
use crate::protocol::packets::packet_component::LoginPropertyElement;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::StateBasedDeserializer;
use crate::protocol::status::status_components::StatusResponseSpec;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;

pub mod packet_component;
mod packet_testing;

/*
This file defines the packets for the most recent supported version of the Protocol

It has a couple of key responsibilities:
- Define the packets that are used in the protocol
- Define the serialization and deserialization for each packet
- Provide vital information about each packet such as the packet ID, the packet state, and the packet direction
*/

// https://wiki.vg/Protocol
// TODO: https://stackoverflow.com/questions/33999341/generating-documentation-in-macros
// TODO: naming it v1_20 would not be forwards compatible? 
packets!(V1_20 => {
	// HANDSHAKE
	Handshaking, HandshakingBody, 0x00, HANDSHAKING, SERVER => {
		protocol_version: VarInt,
		server_address: String,
		port: u16,
		next_state: VarInt
	},
	
	// STATUS
	
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
	}
});