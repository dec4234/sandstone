//! This file defines the packets for the most recent supported version of the Protocol
//! 
//! It has a couple of key responsibilities:
//! - Define the packets that are used in the protocol
//! - Define the serialization and deserialization for each packet
//! - Provide vital information about each packet such as the packet ID, the packet state, and the packet destination
//! 
//! All information for the packets is derived from the wiki. Consider supporting the wiki efforts.
//! https://minecraft.wiki/w/Java_Edition_protocol

use crate::game::player::PlayerGamemode;
use crate::packets;
use crate::protocol::game::info::registry::RegistryDataPacketInternal;
use crate::protocol::game::world::chunk::{ChunkData, LightData};
use crate::protocol::packets::packet_component::{AddResourcePackSpec, LoginCookieResponseSpec, LoginPluginSpec, ResourcePackEntry, TagArray};
use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional, ProtocolPropertyElement};
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::StateBasedDeserializer;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol::status::status_components::StatusResponseSpec;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::game_types::Position;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::util::java::bitfield::BitField;
use sandstone_derive::mc;
use uuid::Uuid;

pub mod packet_component;
pub mod packet_definer;

// https://minecraft.wiki/w/Java_Edition_protocol
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
				uuid: Uuid,
				username: String,
				array: PrefixedArray<ProtocolPropertyElement>
			},
			SetCompression, SetCompressionPacket, 0x03 => {
				#[doc = "The threshold for compression, in bytes. If the packet size is larger than this, it will be compressed."]
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
			LoginStart, LoginStartPacket, 0x00 #[doc = "Initiate the login procedure for a client."] => {
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
			ClientboundPluginMessage, ClientboundPluginMessagePacket, 0x01 => {
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
				packet: RegistryDataPacketInternal
			},
			RemoveResourcePack, RemoveResourcePackPacket, 0x08 => {
				uuid: PrefixedOptional<Uuid>
			},
			AddResourcePack, AddResourcePackPacket, 0x09 => {
				spec: AddResourcePackSpec
			},
			
			// TODO: others here
			
			FeatureFlags, FeatureFlagsPacket, 0x0C => {
				total: VarInt,
				flags: Vec<String>
			},
			UpdateTags, UpdateTagsPacket, 0x0D => {
				tags: PrefixedArray<TagArray>
			},
			ClientboundKnownPacks, ClientboundKnownPacksPacket, 0x0E => {
				entries: PrefixedArray<ResourcePackEntry>
			}
		},
		SERVER => {
			ClientInformation, ClientInformationPacket, 0x00 => {
				locale: String,
				view_distance: i8,
				chat_mode: VarInt,
				chat_colors: bool,
				displayed_skin_parts: u8,
				main_hand: VarInt,
				enable_text_filtering: bool,
				allow_server_listing: bool,
				particle_status: VarInt
			},
			CookieResponse, CookieResponsePacket, 0x01 => {
				key: String,
				payload: PrefixedOptional<PrefixedArray<u8>>
			},
			ServerboundPluginMessage, ServerboundPluginMessagePacket, 0x02 => {
				channel: String,
				data: Vec<u8>
			},
			AcknowledgeFinishConfiguration, AcknowledgeFinishConfigurationPacket, 0x03 => {
				// none
			},
			ServerboundKnownPacks, ServerboundKnownPacksPacket, 0x07 => {
				entries: PrefixedArray<ResourcePackEntry>
			}
		}
	},
	PLAY => {
		CLIENT => {
			ChunkDataUpdateLight, ChunkDataUpdateLightPacket, 0x27 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data_and_Update_Light"] => {
				x: i32,
				y: i32,
				#[doc = "Chunk byte data"]
				data: ChunkData,
				light: LightData
			},
			LoginInfo, LoginInfoPacket, 0x2B => {
				#[doc = "The entity ID of the player. This must remain consistent throughout the session."]
				entity_id: i32,
				is_hardcore: bool,
				dimension_names: PrefixedArray<String>,
				max_players: VarInt,
				#[doc = "Render distance of the server in chunks. The client may use any value less than or equal to this value."]
				render_distance: VarInt,
				simulation_distance: VarInt,
				reduced_debug_info: bool,
				enable_respawn_screen: bool,
				do_limited_crafting: bool,
				dimension_type: VarInt,
				dimension_name: String,
				hashed_seed: i64,
				gamemode: PlayerGamemode,
				previous_gamemode: PlayerGamemode,
				#[doc = "When the world is a debug world"]
				is_debug: bool,
				#[doc = "When the world is superflat"]
				is_flat: bool,
				#[doc = "When true, saves details about the player's death location."]
				has_death_location: bool,
				#[mc(deserialize_if = has_death_location)]
				death_dimension_name: Option<String>,
				#[mc(deserialize_if = has_death_location)]
				death_location: Option<Position>,
				portal_cooldown: VarInt,
				sea_level: VarInt,
				enforces_secure_chat: bool
			},
			PlayerInfoUpdate, PlayerInfoUpdatePacket, 0x3F => {
					
			},
			SyncPlayerPosition, SyncPlayerPositionPacket, 0x41 => {
				teleport_id: VarInt,
				x: f64,
				y: f64,
				z: f64,
				velocity_x: f64,
				velocity_y: f64,
				velocity_z: f64,
				yaw: f32,
				pitch: f32,
				#[doc = "See https://minecraft.wiki/w/Java_Edition_protocol/Data_types#Teleport_Flags for more info"]
				flags: BitField<i8>
			},
			SetCenterChunk, SetCenterChunkPacket, 0x57 => {
				x: VarInt,
				z: VarInt
			}
		},
		SERVER => {
			ConfirmTeleport, ConfirmTeleportPacket, 0x00 => {
				teleport_id: VarInt
			},
			SetPlayerPositionRotation, SetPlayerPositionRotationPacket, 0x1D => {
				x: f64,
				#[doc = "Feet y position. Head - 1.62"]
				y: f64,
				z: f64,
				yaw: f32,
				pitch: f32,
				flags: BitField<i8>
			}
		}
	}
});
