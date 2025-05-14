//! This file defines the packets for the most recent supported version of the Protocol
//! 
//! It has a couple of key responsibilities:
//! - Define the packets that are used in the protocol
//! - Define the serialization and deserialization for each packet
//! - Provide vital information about each packet such as the packet ID, the packet state, and the packet destination
//! 
//! All information for the packets is derived from the wiki. Consider supporting the wiki efforts.
//! https://minecraft.wiki/w/Java_Edition_protocol

use uuid::Uuid;

use crate::packets;
use crate::protocol::packets::packet_component::{AddResourcePackSpec, LoginCookieResponseSpec, LoginPluginSpec, RegistryEntry, RemoveResourcePackSpec};
use crate::protocol::packets::packet_component::LoginPropertyElement;
use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::StateBasedDeserializer;
use crate::protocol::status::status_components::StatusResponseSpec;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::var_types::VarInt;

pub mod packet_component;
pub mod packet_definer;

// https://minecraft.wiki/w/Java_Edition_protocol
// TODO: https://stackoverflow.com/questions/33999341/generating-documentation-in-macros
packets!(v1_21 => { // version name is for reference only, has no effect
	HANDSHAKING => {
		SERVER => { // Server is the destination
			Handshaking, HandshakingPacket, 0x00 #[doc = "Used to switch server into a different connection state. Should be sent shortly after TCP connection is opened."] => {
				#[doc = "See protocol_version.rs for more context"]
				protocol_version: VarInt,
				#[doc = "The server address, in the form of a domain name or IP address"]
				server_address: String,
				port: u16,
				#[doc = "1 for STATUS, 2 for LOGIN, 3 for TRANSFER"]
				next_state: VarInt
			}
		}
	},
	STATUS => {
		CLIENT => {
			StatusResponse, StatusResponsePacket, 0x00 => {
				response: StatusResponseSpec
			},
			PingResponse, PingResponsePacket, 0x01 => {
				payload: u64
			}
		},
		SERVER => {
			StatusRequest, StatusRequestPacket, 0x00 => {
				// none
			},
			PingRequest, PingRequestPacket, 0x01 => {
				payload: i64
			}
		}
	},
	LOGIN => {
		CLIENT => {
			Disconnect, DisconnectPacket, 0x00 => {
				reason: TextComponent
			},
			EncryptionRequest, EncryptionRequestPacket, 0x01 => {
				server_id: String,
				public_key_length: VarInt,
				public_key: Vec<u8>,
				verify_token_length: VarInt, // always 4 for Notchian servers
				verify_token: Vec<u8>
			},
			LoginSuccess, LoginSuccessPacket, 0x02 => {
				uuid: String,
				username: String,
				num_properties: VarInt,
				array: Vec<LoginPropertyElement>,
				strict_error_handling: bool
			},
			SetCompression, SetCompressionPacket, 0x03 => {
				threshold: VarInt
			},
			LoginPluginRequest, LoginPluginRequestPacket, 0x04 => {
				message_id: VarInt,
				channel: String,
				data: Vec<u8>
			},
			LoginCookieRequest, LoginCookieRequestPacket, 0x05 => {
				key: String
			}
		},
		SERVER => {
			LoginStart, LoginStartPacket, 0x00 => {
				username: String,
				uuid: Uuid
			},
			EncryptionResponse, EncryptionResponsePacket, 0x01 => {
				shared_secret_length: VarInt,
				shared_secret: Vec<u8>,
				verify_token_length: VarInt,
				verify_token: Vec<u8>
			},
			LoginPluginResponse, LoginPluginResponsePacket, 0x02 => {
				response: LoginPluginSpec
			},
			LoginAcknowledged, LoginAcknowledgedPacket, 0x03 => {
				// none
			},
			LoginCookieResponse, LoginCookieResponsePacket, 0x04 => {
				spec: LoginCookieResponseSpec
			}
		}
	},
	CONFIGURATION => {
		CLIENT => {
			ConfigCookieRequest, ConfigCookieRequestPacket, 0x00 => {
				key: String
			},
			PluginMessage, PluginMessagePacket, 0x01 => {
				channel: String,
				data: Vec<u8>
			},
			ConfigDisconnect, ConfigDisconnectPacket, 0x02 => {
				reason: TextComponent
			},
			FinishConfiguration, FinishConfigurationPacket, 0x03 => {
				// none
			},
			KeepAlive, KeepAlivePacket, 0x04 => {
				keep_alive_id: i64
			},
			ConfigurationPing, ConfigurationPingPacket, 0x05 => {
				payload: i32
			},
			ResetChat, ResetChatPacket, 0x06 => {
				// none
			},
			RegistryData, RegistryDataPacket, 0x07 => {
				registry_id: String,
				entry_count: VarInt,
				entries: Vec<RegistryEntry>
			},
			RemoveResourcePack, RemoveResourcePackPacket, 0x08 => {
				spec: RemoveResourcePackSpec
			},
			AddResourcePack, AddResourcePackPacket, 0x09 => {
				spec: AddResourcePackSpec
			},
			
			// TODO: others here
			
			FeatureFlags, FeatureFlagsPacket, 0x0C => {
				total: VarInt,
				flags: Vec<String>
			}
		}
	}
});
