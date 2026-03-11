//! This file defines the packets for the most recent supported version of the Protocol
//!
//! It has a couple of key responsibilities:
//! - Define the packets that are used in the protocol
//! - Define the serialization and deserialization for each packet
//! - Provide vital information about each packet such as the packet ID, the packet state, and the packet destination
//!
//! All information for the packets is derived from the wiki. Consider supporting the wiki efforts.
//! https://minecraft.wiki/w/Java_Edition_protocol

#![allow(clippy::too_many_arguments)]
#![allow(clippy::new_without_default)] // todo: maybe needs default?

use crate::game::player::PlayerGamemode;
use crate::packets;
use crate::protocol::game::effects::sound::{SoundCategory, SoundEvent};
use crate::protocol::game::entity::EntityMetadata;
use crate::protocol::game::info::inventory::slotdata::SlotData;
use crate::protocol::game::info::registry::RegistryDataPacketInternal;
use crate::protocol::game::player::player_action::PlayerInfoUpdateData;
use crate::protocol::game::player::{ClientStatusAction, RespawnKeptData};
use crate::protocol::game::world::chunk::{ChunkData, LightData};
use crate::protocol::packets::packet_component::{AddResourcePackSpec, AttributeProperty, EquipmentList, GameEventType, LoginCookieResponseSpec, LoginPluginSpec, PlayerAbilityFlags, PropertySet, RecipeBookEntry, ResourcePackEntry, StonecutterRecipe, Tag};
use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::StateBasedDeserializer;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol::status::status_components::StatusResponseSpec;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::command::Node;
use crate::protocol_types::datatypes::game_types::{ChunkSectionPosition, GameDifficulty, Position, SectionBlockEntry, SourcePosition, WorldEventType};
use crate::protocol_types::datatypes::internal_types::{Angle, IDorX, LpVec3, Mapping};
use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};
use crate::util::java::bitfield::BitField;
use packet_component::ProtocolPropertyElement;
use uuid::Uuid;

pub mod packet_component;
pub mod packet_definer;

// https://minecraft.wiki/w/Java_Edition_protocol
packets!(v1_21 => { // version name is for reference only, has no effect
	HANDSHAKING => {
		SERVER => { // Server is the destination
			Handshaking, 0x00 #[doc = "Used to switch server into a different connection state. Should be sent shortly after TCP connection is opened."] => {
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
			StatusResponse, 0x00 => {
				response: StatusResponseSpec
			},
			PingResponse, 0x01 => {
				payload: u64
			}
		},
		SERVER => {
			StatusRequest, 0x00 => {
				// none
			},
			PingRequest, 0x01 => {
				payload: i64
			}
		}
	},
	LOGIN => {
		CLIENT => {
			LoginDisconnect, 0x00 => {
				reason: TextComponent
			},
			EncryptionRequest, 0x01 => {
				server_id: String,
				public_key_length: VarInt,
				public_key: Vec<u8>,
				verify_token_length: VarInt, // always 4 for Notchian servers
				verify_token: Vec<u8>
			},
			LoginSuccess, 0x02 => {
				uuid: Uuid,
				username: String,
				array: PrefixedArray<ProtocolPropertyElement>
			},
			SetCompression, 0x03 => {
				#[doc = "The threshold for compression, in bytes. If the packet size is larger than this, it will be compressed."]
				threshold: VarInt
			},
			LoginPluginRequest, 0x04 => {
				message_id: VarInt,
				channel: String,
				data: Vec<u8>
			},
			LoginCookieRequest, 0x05 => {
				key: String
			}
		},
		SERVER => {
			LoginStart, 0x00 #[doc = "Initiate the login procedure for a client."] => {
				username: String,
				uuid: Uuid
			},
			EncryptionResponse, 0x01 => {
				shared_secret_length: VarInt,
				shared_secret: Vec<u8>,
				verify_token_length: VarInt,
				verify_token: Vec<u8>
			},
			LoginPluginResponse, 0x02 => {
				response: LoginPluginSpec
			},
			LoginAcknowledged, 0x03 => {
				// none
			},
			LoginCookieResponse, 0x04 => {
				spec: LoginCookieResponseSpec
			}
		}
	},
	CONFIGURATION => {
		CLIENT => {
			ConfigCookieRequest, 0x00 => {
				key: String
			},
			ClientboundPluginMessage, 0x01 => {
				channel: String,
				data: Vec<u8>
			},
			ConfigDisconnect, 0x02 => {
				reason: TextComponent
			},
			FinishConfiguration, 0x03 => {
				// none
			},
			KeepAlive, 0x04 => {
				keep_alive_id: i64
			},
			ConfigurationPing, 0x05 => {
				payload: i32
			},
			ResetChat, 0x06 => {
				// none
			},
			RegistryData, 0x07 => {
				packet: RegistryDataPacketInternal
			},
			RemoveResourcePack, 0x08 => {
				uuid: PrefixedOptional<Uuid>
			},
			AddResourcePack, 0x09 => {
				spec: AddResourcePackSpec
			},

			// TODO: others here

			FeatureFlags, 0x0C => {
				total: VarInt,
				flags: Vec<String>
			},
			UpdateTags, 0x0D => {
				tags: PrefixedArray<Mapping<PrefixedArray<Tag>>>
			},
			ClientboundKnownPacks, 0x0E => {
				entries: PrefixedArray<ResourcePackEntry>
			}
		},
		SERVER => {
			ClientInformation, 0x00 => {
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
			CookieResponse, 0x01 => {
				key: String,
				payload: PrefixedOptional<PrefixedArray<u8>>
			},
			ServerboundPluginMessage, 0x02 => {
				channel: String,
				data: Vec<u8>
			},
			AcknowledgeFinishConfiguration, 0x03 => {
				// none
			},
			ServerboundKnownPacks, 0x07 => {
				entries: PrefixedArray<ResourcePackEntry>
			}
		}
	},
	PLAY => {
		CLIENT => {
			BundleDelimiter, 0x00 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Bundle_Delimiter"] => {
				// no fields
			},
			SpawnEntity, 0x01 => {
				entity_id: VarInt,
				entity_uuid: Uuid,
				typ: VarInt,
				x: f64,
				y: f64,
				z: f64,
				velocity: LpVec3,
				pitch: Angle,
				yaw: Angle,
				head_yaw: Angle,
				data: VarInt
			},
			BlockUpdate, 0x08 => {
				location: Position,
				block_id: VarInt
			},
			ChangeDifficulty, 0x0A => {
				difficulty: GameDifficulty,
				difficulty_locked: bool
			},
			ChunkBatchFinished, 0x0B => {
				size: VarInt
			},
			ChunkBatchStart, 0x0C => {
				// no fields
			},
			CommandsGraph, 0x10 => {
				nodes: PrefixedArray<Node>,
				root_index: VarInt
			},
			SetContainerContent, 0x12 => {
				window_id: VarInt,
				state_id: VarInt,
				slot_data: PrefixedArray<SlotData>,
				carried_item: SlotData
			},
			DamageEvent, 0x19 => {
				entity_id: VarInt,
				source_type_id: VarInt,
				source_cause_id: VarInt,
				source_direct_id: VarInt,
				source_position: PrefixedOptional<SourcePosition>
			},
			DisconnectPlay, 0x20 => {
				reason: TextComponent
			},
			EntityEvent, 0x22 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Entity_statuses"] => {
				entity_id: i32,
				entity_status: i8 // todo: create comprehensive enum
			},
			TeleportEntity, 0x23 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Teleport_Entity"] => {
				entity_id: VarInt,
				x: f64,
				y: f64,
				z: f64,
				velocity_x: f64,
				velocity_y: f64,
				velocity_z: f64,
				yaw: f32,
				pitch: f32,
				on_ground: bool
			},
			GameEvent, 0x26 => {
				event: GameEventType,
				value: f32
			},
			InitializeWorldBorder, 0x2A => {
				x: f64,
				z: f64,
				old_diameter: f64,
				new_diameter: f64,
				speed: VarLong,
				portal_teleport_boundary: VarInt,
				warning_blocks: VarInt,
				warning_time: VarInt
			},
			ClientboundKeepAlive, 0x2B => {
				keep_alive_id: i64
			},
			ChunkDataUpdateLight, 0x2C #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data_and_Update_Light"] => {
				x: i32,
				z: i32,
				#[doc = "Chunk byte data"]
				data: ChunkData,
				light: LightData
			},
			WorldEvent, 0x2D #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#World_Event"] => {
				event: WorldEventType,
				location: Position,
				data: i32,
				disable_relative_volume: bool
			},
			UpdateLight, 0x2F => {
				x_chunk: VarInt,
				y_chunk: VarInt,
				data: LightData
			},
			LoginInfo, 0x30 => {
				#[doc = "The entity ID of the player. This must remain consistent throughout the session."]
				entity_id: i32,
				is_hardcore: bool,
				dimension_names: PrefixedArray<String>,
				max_players: VarInt,
				#[doc = "Render distance of the server in chunks. The client may use any value less than or equal to this value."]
				view_distance: VarInt,
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
			UpdateEntityPosition, 0x33 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Entity_Position"] => {
				entity_id: VarInt,
				#[doc = "Change in X position as currentX * 4096 - prevX * 4096"]
				x_delta: i16,
				#[doc = "Change in Y position as currentY * 4096 - prevY * 4096"]
				y_delta: i16,
				#[doc = "Change in Z position as currentZ * 4096 - prevZ * 4096"]
				z_delta: i16,
				on_ground: bool
			},
			UpdateEntityPostitionRotation, 0x34 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Entity_Position_and_Rotation"] => {
				entity_id: VarInt,
				#[doc = "Change in X position as currentX * 4096 - prevX * 4096"]
				x_delta: i16,
				#[doc = "Change in Y position as currentY * 4096 - prevY * 4096"]
				y_delta: i16,
				#[doc = "Change in Z position as currentZ * 4096 - prevZ * 4096"]
				z_delta: i16,
				yaw: Angle,
				pitch: Angle,
				on_ground: bool
			},
			UpdateEntityRotation, 0x36 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Entity_Rotation"] => {
				entity_id: VarInt,
				yaw: Angle,
				pitch: Angle,
				on_ground: bool
			},
			PlayerAbilities, 0x3E => {
				flags: PlayerAbilityFlags,
				flying_speed: f32,
				fov_modifier: f32
			},
			PlayerInfoUpdate, 0x44 => {
				data: PlayerInfoUpdateData
			},
			SyncPlayerPosition, 0x46 => {
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
			RecipeBookAdd, 0x48 => {
				recipes: PrefixedArray<RecipeBookEntry>,
				replace: bool
			},
			RecipeBookRemove, 0x49 => {
				recipes: PrefixedArray<VarInt>
			},
			RecipeBookSettings, 0x4A => {
				crafting_open: bool,
				crafting_filter: bool,
				smelting_open: bool,
				smelting_filter: bool,
				blasting_open: bool,
				blasting_filter: bool,
				smoking_open: bool,
				smoking_filter: bool
			},
			RemoveEntities, 0x4B => {
				#[doc = "A prefixed array of entity ids to be destroyed"]
				entities: PrefixedArray<VarInt>
			},
			Respawn, 0x50 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Respawn"] => {
				dimension_type: VarInt,
				dimension_name: String,
				hashed_seed: i64,
				gamemode: PlayerGamemode,
				previous_gamemode: PlayerGamemode,
				is_debug: bool,
				is_flat: bool,
				has_death_location: bool,
				death_dimension_name: Option<String>,
				death_location: Option<Position>,
				portal_cooldown: VarInt,
				sea_level: VarInt,
				data_kept: RespawnKeptData
			},
			SetHeadRotation, 0x51 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Head_Rotation"] => {
				entity_id: VarInt,
				head_yaw: Angle
			},
			SectionBlocksUpdate, 0x52 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Section_Blocks_Update"] => {
				chunk_section_position: ChunkSectionPosition,
				blocks: PrefixedArray<SectionBlockEntry>
			},
			ServerData, 0x54 => {
				motd: TextComponent,
				icon: PrefixedOptional<PrefixedArray<u8>>
			},
			SetCenterChunk, 0x5C => {
				x: VarInt,
				z: VarInt
			},
			SetDefaultSpawnPosition, 0x5F => {
				dimension_name: String,
				location: Position,
				yaw: f32,
				pitch: f32
			},
			SetEntityMetadata, 0x61 => {
				entity_id: VarInt,
				metadata: EntityMetadata
			},
			LinkEntries, 0x62 => {
				attached_id: i32,
				holding_id: i32
			},
			SetEntityVelocity, 0x63 => {
				entity_id: VarInt,
				velocity: LpVec3
			},
			SetEquipment, 0x64 => {
				entity_id: VarInt,
				equipment: EquipmentList
			},
			SetExperience, 0x65 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Experience"] => {
				experience_bar: f32,
				level: VarInt,
				total_experience: VarInt
			},
			SetHealth, 0x66 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Health"] => {
				health: f32,
				food: VarInt,
				saturation: f32
			},
			SetHeldItem, 0x67 => {
				slot: VarInt
			},
			SetPassengers, 0x69 => {
				entity_id: VarInt,
				passengers: PrefixedArray<VarInt>
			},
			UpdateTime, 0x6F => {
				world_age: i64,
				time_of_day: i64,
				time_of_day_increasing: bool
			},
			SoundEffect, 0x73 => {
				sound_event: IDorX<SoundEvent>,
				sound_category: SoundCategory,
				entity_id: VarInt,
				volume: f32,
				pitch: f32,
				seed: i64
			},
			SetTickingState, 0x7D => {
				tick_rate: f32,
				is_frozen: bool
			},
			StepTick, 0x7E => {
				tick_steps: VarInt
			},
			UpdateAdvancements, 0x80 => {
				unimplemented: Vec<u8> // todo: fix UpdateAdvancements
				/*reset: bool,
				advancement_mapping: PrefixedArray<Mapping<Advancement>>,
				identifiers: PrefixedArray<String>,
				progress_mapping: PrefixedArray<Mapping<PrefixedArray<Mapping<PrefixedOptional<i64>>>>>,
				show_advancements: bool*/
			},
			UpdateAttributes, 0x81 => {
				entity_id: VarInt,
				modifiers: PrefixedArray<AttributeProperty>
			},
			UpdateRecipes, 0x83 => {
				property_sets: PrefixedArray<PropertySet>,
				stonecutter_recipes: PrefixedArray<StonecutterRecipe>
			}
		},
		SERVER => {
			ConfirmTeleport, 0x00 => {
				teleport_id: VarInt
			},
			ClientCommand, 0x0B #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Command"] => {
				action: ClientStatusAction
			},
			ServerboundKeepAlive, 0x1B => {
				keep_alive_id: i64
			},
			SetPlayerPositionRotation, 0x1D => {
				x: f64,
				#[doc = "Feet y position. Head - 1.62"]
				y: f64,
				z: f64,
				yaw: f32,
				pitch: f32,
				flags: BitField<i8>
			},
			PlayerLoaded, 0x2B => {
				// none
			},
			ResourcePackResponse, 0x30 => {
				uuid: Uuid,
				result: VarInt
			}
		}
	}
});