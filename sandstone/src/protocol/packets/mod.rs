//! This file defines the packets for the most recent supported version of the Protocol
//! 
//! It has a couple of key responsibilities:
//! - Define the packets that are used in the protocol
//! - Define the serialization and deserialization for each packet
//! - Provide vital information about each packet such as the packet ID, the packet state, and the packet direction

use uuid::Uuid;

use crate::packets;
use crate::protocol::packet_definer::{PacketDirection, PacketState};
use crate::protocol::packets::packet_component::{AddResourcePackSpec, LoginCookieResponseSpec, LoginPluginSpec, RegistryEntry, RemoveResourcePackSpec};
use crate::protocol::packets::packet_component::LoginPropertyElement;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::StateBasedDeserializer;
use crate::protocol::status::status_components::StatusResponseSpec;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::var_types::VarInt;

pub mod packet_component;

// https://wiki.vg/Protocol
// TODO: https://stackoverflow.com/questions/33999341/generating-documentation-in-macros
packets!(v1_21 => { // version name is for reference only, has no effect
	HANDSHAKING => {
		SERVER => {
			Handshaking, HandshakingBody, 0x00 => {
				protocol_version: VarInt,
				server_address: String,
				port: u16,
				next_state: VarInt
			}
		}
	},
	STATUS => {
		CLIENT => {
			StatusResponse, StatusResponseBody, 0x00 => {
				response: StatusResponseSpec
			},
			PingResponse, PingResponseBody, 0x01 => {
				payload: u64
			}
		},
		SERVER => {
			StatusRequest, StatusRequestBody, 0x00 => {
				// none
			},
			PingRequest, PingRequestBody, 0x01 => {
				payload: i64
			}
		}
	},
	LOGIN => {
		CLIENT => {
			Disconnect, DisconnectBody, 0x00 => {
				reason: TextComponent
			},
			EncryptionRequest, EncryptionRequestBody, 0x01 => {
				server_id: String,
				public_key_length: VarInt,
				public_key: Vec<u8>,
				verify_token_length: VarInt, // always 4 for Notchian servers
				verify_token: Vec<u8>
			},
			LoginSuccess, LoginSuccessBody, 0x02 => {
				uuid: String,
				username: String,
				num_properties: VarInt,
				array: Vec<LoginPropertyElement>,
				strict_error_handling: bool
			},
			SetCompression, SetCompressionBody, 0x03 => {
				threshold: VarInt
			},
			LoginPluginRequest, LoginPluginRequestBody, 0x04 => {
				message_id: VarInt,
				channel: String,
				data: Vec<u8>
			},
			LoginCookieRequest, LoginCookieRequestBody, 0x05 => {
				key: String
			}
		},
		SERVER => {
			LoginStart, LoginStartBody, 0x00 => {
				username: String,
				uuid: Uuid
			},
			EncryptionResponse, EncryptionResponseBody, 0x01 => {
				shared_secret_length: VarInt,
				shared_secret: Vec<u8>,
				verify_token_length: VarInt,
				verify_token: Vec<u8>
			},
			LoginPluginResponse, LoginPluginResponseBody, 0x02 => {
				response: LoginPluginSpec
			},
			LoginAcknowledged, LoginAcknowledgedBody, 0x03 => {
				// none
			},
			LoginCookieResponse, LoginCookieResponseBody, 0x04 => {
				spec: LoginCookieResponseSpec
			}
		}
	},
	CONFIGURATION => {
		CLIENT => {
			ConfigCookieRequest, ConfigCookieRequestBody, 0x00 => {
				key: String
			},
			PluginMessage, PluginMessageBody, 0x01 => {
				channel: String,
				data: Vec<u8>
			},
			ConfigDisconnect, ConfigDisconnectBody, 0x02 => {
				reason: TextComponent
			},
			FinishConfiguration, FinishConfigurationBody, 0x03 => {
				// none
			},
			KeepAlive, KeepAliveBody, 0x04 => {
				keep_alive_id: i64
			},
			ConfigurationPing, ConfigurationPingBody, 0x05 => {
				payload: i32
			},
			ResetChat, ResetChatBody, 0x06 => {
				// none
			},
			RegistryData, RegistryDataBody, 0x07 => {
				registry_id: String,
				entry_count: VarInt,
				entries: Vec<RegistryEntry>
			},
			RemoveResourcePack, RemoveResourcePackBody, 0x08 => {
				spec: RemoveResourcePackSpec
			},
			AddResourcePack, AddResourcePackBody, 0x09 => {
				spec: AddResourcePackSpec
			},
			
			// TODO: others here
			
			FeatureFlags, FeatureFlagsBody, 0x0C => {
				total: VarInt,
				flags: Vec<String>
			}
		}
	}
});