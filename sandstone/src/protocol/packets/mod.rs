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
use crate::protocol::packets::packet_component::{
	AddResourcePackSpec, AttributeProperty, BossBarUpdateAction, ChunkBiomeData, CustomReportDetails, EquipmentList, GameEventType, InteractHand, InteractType, LoginCookieResponseSpec,
	LoginPluginSpec, PlayerAbilityFlags, PlayerInputFlags, PlayerPositionFlags, PropertySet, RecipeBookEntry, ResourcePackEntry, ServerLink, StatisticAward, StonecutterRecipe, Tag, TooltipMatch,
};
use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::StateBasedDeserializer;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol::status::status_components::StatusResponseSpec;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::{JsonTextComponent, TextComponent};
use crate::protocol_types::datatypes::command::Node;
use crate::protocol_types::datatypes::game_types::{ChunkSectionPosition, GameDifficulty, Position, SectionBlockEntry, SourcePosition, WorldEventType};
use crate::protocol_types::datatypes::internal_types::{Angle, IDorX, LpVec3, Mapping};
use crate::protocol_types::datatypes::nbt::nbt::{NbtCompound, NbtTag};
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
				reason: JsonTextComponent
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
			ConfigStoreCookie, 0x0A => {
				key: String,
				payload: PrefixedArray<u8>
			},
			ConfigTransfer, 0x0B => {
				host: String,
				port: VarInt
			},
			FeatureFlags, 0x0C => {
				total: VarInt,
				flags: Vec<String>
			},
			UpdateTags, 0x0D => {
				tags: PrefixedArray<Mapping<PrefixedArray<Tag>>>
			},
			ClientboundKnownPacks, 0x0E => {
				entries: PrefixedArray<ResourcePackEntry>
			},
			ConfigCustomReportDetails, 0x0F => {
				details: PrefixedArray<CustomReportDetails>
			},
			ConfigServerLinks, 0x10 => {
				links: PrefixedArray<ServerLink>
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
			ServerboundKeepAliveConfig, 0x04 => {
				keep_alive_id: i64
			},
			ConfigPong, 0x05 => {
				id: i32
			},
			ConfigResourcePackResponse, 0x06 => {
				uuid: Uuid,
				result: VarInt
			},
			ServerboundKnownPacks, 0x07 => {
				entries: PrefixedArray<ResourcePackEntry>
			},
			ConfigCustomClickAction, 0x08 => {
				id: String,
				payload: NbtCompound
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
			EntityAnimation, 0x02 => {
				entity_id: VarInt,
				animation: u8
			},
			AwardStatistics, 0x03 => {
				stats: PrefixedArray<StatisticAward>
			},
			AcknowledgeBlockChange, 0x04 => {
				sequence_id: VarInt
			},
			SetBlockDestroyStage, 0x05 => {
				entity_id: VarInt,
				location: Position,
				#[doc = "0-9 to set, any other value removes the stage"]
				destroy_stage: i8
			},
			BlockEntityData, 0x06 => {
				location: Position,
				typ: VarInt,
				data: NbtTag
			},
			BlockAction, 0x07 => {
				location: Position,
				action_id: u8,
				action_param: u8,
				block_type: VarInt
			},
			BlockUpdate, 0x08 => {
				location: Position,
				block_id: VarInt
			},
			BossBar, 0x09 => {
				uuid: Uuid,
				action: BossBarUpdateAction
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
			ChunkBiomes, 0x0D => {
				chunk_biome_data: PrefixedArray<ChunkBiomeData>
			},
			ClearTitles, 0x0E => {
				reset: bool
			},
			CommandSuggestionsResponse, 0x0F => {
				id: VarInt,
				start: VarInt,
				length: VarInt,
				matches: PrefixedArray<TooltipMatch>
			},
			CommandsGraph, 0x10 => {
				nodes: PrefixedArray<Node>,
				root_index: VarInt
			},
			CloseContainer, 0x11 => {
				window_id: VarInt
			},
			SetContainerContent, 0x12 => {
				window_id: VarInt,
				state_id: VarInt,
				slot_data: PrefixedArray<SlotData>,
				carried_item: SlotData
			},
			SetContainerProperty, 0x13 => {
				window_id: VarInt,
				property: i16,
				value: i16
			},
			SetContainerSlot, 0x14 => {
				window_id: VarInt,
				state_id: VarInt,
				slot: i16,
				slot_data: SlotData
			},
			CookieRequestPlay, 0x15 => {
				key: String
			},
			SetCooldown, 0x16 => {
				cooldown_group: String,
				cooldown_ticks: VarInt
			},
			ChatSuggestions, 0x17 => {
				action: VarInt,
				entries: PrefixedArray<String>
			},
			ClientboundPluginMessagePlay, 0x18 => {
				channel: String,
				data: Vec<u8>
			},
			DamageEvent, 0x19 => {
				entity_id: VarInt,
				source_type_id: VarInt,
				source_cause_id: VarInt,
				source_direct_id: VarInt,
				source_position: PrefixedOptional<SourcePosition>
			},
			DebugBlockValue, 0x1A => {
				// TODO
			},
			DebugChunkValue, 0x1B => {
				// TODO
			},
			DebugEntityValue, 0x1C => {
				// TODO
			},
			DebugEvent, 0x1D => {
				// TODO
			},
			DebugSample, 0x1E => {
				// TODO
			},
			DeleteMessage, 0x1F => {
				// TODO: message id + optional signature
			},
			DisconnectPlay, 0x20 => {
				reason: TextComponent
			},
			DisguisedChatMessage, 0x21 => {
				// TODO: chat type with branching fields
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
			Explosion, 0x24 => {
				// TODO: affected blocks array + optional particle/sound data
			},
			UnloadChunk, 0x25 => {
				#[doc = "Note: Chunk Z is sent before Chunk X"]
				z: i32,
				x: i32
			},
			GameEvent, 0x26 => {
				event: GameEventType,
				value: f32
			},
			GameTestHighlightPosition, 0x27 => {
				// TODO
			},
			OpenHorseScreen, 0x28 => {
				window_id: VarInt,
				slot_count: VarInt,
				entity_id: i32
			},
			HurtAnimation, 0x29 => {
				entity_id: VarInt,
				yaw: f32
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
			Particle, 0x2E => {
				// TODO: particle id + branching particle data
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
			MapData, 0x31 => {
				// TODO: icons array + color patch data
			},
			MerchantOffers, 0x32 => {
				// TODO: trade offers array
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
			MoveMinecartAlongTrack, 0x35 => {
				// TODO: array of movement steps
			},
			UpdateEntityRotation, 0x36 #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Entity_Rotation"] => {
				entity_id: VarInt,
				yaw: Angle,
				pitch: Angle,
				on_ground: bool
			},
			MoveVehicle, 0x37 => {
				x: f64,
				y: f64,
				z: f64,
				yaw: f32,
				pitch: f32
			},
			OpenBook, 0x38 => {
				hand: VarInt
			},
			OpenScreen, 0x39 => {
				window_id: VarInt,
				window_type: VarInt,
				window_title: TextComponent
			},
			OpenSignEditor, 0x3A => {
				location: Position,
				is_front_text: bool
			},
			PingPlay, 0x3B => {
				id: i32
			},
			PingResponsePlay, 0x3C => {
				payload: i64
			},
			PlaceGhostRecipe, 0x3D => {
				// TODO: window id + recipe display
			},
			PlayerAbilities, 0x3E => {
				flags: PlayerAbilityFlags,
				flying_speed: f32,
				fov_modifier: f32
			},
			PlayerChatMessage, 0x3F => {
				// TODO: signed chat message with branching fields
			},
			EndCombat, 0x40 => {
				duration: VarInt
			},
			EnterCombat, 0x41 => {
				// no fields
			},
			CombatDeath, 0x42 => {
				#[doc = "Entity ID of the player that died (should match the client's entity ID)"]
				id: VarInt,
				message: TextComponent
			},
			PlayerInfoRemove, 0x43 => {
				players: PrefixedArray<Uuid>
			},
			PlayerInfoUpdate, 0x44 => {
				data: PlayerInfoUpdateData
			},
			LookAt, 0x45 => {
				feet_eyes: VarInt,
				target_x: f64,
				target_y: f64,
				target_z: f64,
				is_entity: bool,
				#[mc(deserialize_if = is_entity)]
				entity_id: Option<VarInt>,
				#[mc(deserialize_if = is_entity)]
				entity_feet_eyes: Option<VarInt>
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
				flags: BitField<i32>
			},
			PlayerRotation, 0x47 => {
				#[doc = "Rotation on the X axis, in degrees."]
				yaw: f32,
				relative_yaw: bool,
				#[doc = "Rotation on the Y axis, in degrees."]
				pitch: f32,
				relative_pitch: bool
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
			RemoveEntityEffect, 0x4C => {
				entity_id: VarInt,
				effect_id: VarInt
			},
			ResetScore, 0x4D => {
				entity_name: String,
				objective_name: PrefixedOptional<String>
			},
			RemoveResourcePackPlay, 0x4E => {
				uuid: PrefixedOptional<Uuid>
			},
			AddResourcePackPlay, 0x4F => {
				spec: AddResourcePackSpec
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
			SelectAdvancementsTab, 0x53 => {
				tab_id: PrefixedOptional<String>
			},
			ServerData, 0x54 => {
				motd: TextComponent,
				icon: PrefixedOptional<PrefixedArray<u8>>
			},
			SetActionBarText, 0x55 => {
				action_bar_text: TextComponent
			},
			SetBorderCenter, 0x56 => {
				x: f64,
				z: f64
			},
			SetBorderLerpSize, 0x57 => {
				old_diameter: f64,
				new_diameter: f64,
				speed: VarLong
			},
			SetBorderSize, 0x58 => {
				diameter: f64
			},
			SetBorderWarningDelay, 0x59 => {
				warning_delay: VarInt
			},
			SetBorderWarningDistance, 0x5A => {
				warning_distance: VarInt
			},
			SetCamera, 0x5B => {
				camera_entity_id: VarInt
			},
			SetCenterChunk, 0x5C => {
				x: VarInt,
				z: VarInt
			},
			SetRenderDistance, 0x5D => {
				render_distance: VarInt
			},
			SetCursorItem, 0x5E => {
				item: SlotData
			},
			SetDefaultSpawnPosition, 0x5F => {
				dimension_name: String,
				location: Position,
				yaw: f32,
				pitch: f32
			},
			DisplayObjective, 0x60 => {
				position: VarInt,
				score_name: String
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
			UpdateObjectives, 0x68 => {
				// TODO: mode-dependent display name + number format
			},
			SetPassengers, 0x69 => {
				entity_id: VarInt,
				passengers: PrefixedArray<VarInt>
			},
			SetPlayerInventorySlot, 0x6A => {
				slot: VarInt,
				item: SlotData
			},
			UpdateTeams, 0x6B => {
				// TODO: mode-dependent team info
			},
			UpdateScore, 0x6C => {
				// TODO: optional display name + number format
			},
			SetSimulationDistance, 0x6D => {
				simulation_distance: VarInt
			},
			SetSubtitleText, 0x6E => {
				subtitle_text: TextComponent
			},
			UpdateTime, 0x6F => {
				world_age: i64,
				time_of_day: i64,
				time_of_day_increasing: bool
			},
			SetTitleText, 0x70 => {
				title_text: TextComponent
			},
			SetTitleAnimationTimes, 0x71 => {
				fade_in: i32,
				stay: i32,
				fade_out: i32
			},
			EntitySoundEffect, 0x72 => {
				sound_event: IDorX<SoundEvent>,
				sound_category: SoundCategory,
				entity_id: VarInt,
				volume: f32,
				pitch: f32,
				seed: i64
			},
			SoundEffect, 0x73 => {
				sound_event: IDorX<SoundEvent>,
				sound_category: SoundCategory,
				entity_id: VarInt,
				volume: f32,
				pitch: f32,
				seed: i64
			},
			StartConfiguration, 0x74 => {
				// no fields
			},
			StopSound, 0x75 => {
				// TODO: flags byte with conditional source/sound
			},
			StoreCookie, 0x76 => {
				key: String,
				payload: PrefixedArray<u8>
			},
			SystemChatMessage, 0x77 => {
				content: TextComponent,
				overlay: bool
			},
			SetTabListHeaderAndFooter, 0x78 => {
				header: TextComponent,
				footer: TextComponent
			},
			TagQueryResponse, 0x79 => {
				transaction_id: VarInt,
				nbt: NbtTag
			},
			PickupItem, 0x7A => {
				collected_entity_id: VarInt,
				collector_entity_id: VarInt,
				pickup_item_count: VarInt
			},
			SynchronizeVehiclePosition, 0x7B => {
				// TODO: position + velocity + flags
			},
			TestInstanceBlockStatus, 0x7C => {
				// TODO: status with optional size
			},
			SetTickingState, 0x7D => {
				tick_rate: f32,
				is_frozen: bool
			},
			StepTick, 0x7E => {
				tick_steps: VarInt
			},
			Transfer, 0x7F => {
				host: String,
				port: VarInt
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
			EntityEffect, 0x82 => {
				entity_id: VarInt,
				effect_id: VarInt,
				amplifier: i8,
				duration: VarInt,
				flags: i8
			},
			UpdateRecipes, 0x83 => {
				property_sets: PrefixedArray<PropertySet>,
				stonecutter_recipes: PrefixedArray<StonecutterRecipe>
			},
			UpdateTagsPlay, 0x84 => {
				tags: PrefixedArray<Mapping<PrefixedArray<Tag>>>
			},
			ProjectilePower, 0x85 => {
				// TODO
			},
			CustomReportDetails, 0x86 => {
				// TODO: array of structured details
			},
			ServerLinks, 0x87 => {
				links: PrefixedArray<ServerLink>
			},
			Waypoint, 0x88 => {
				// TODO: operation + branching waypoint data
			},
			ClearDialog, 0x89 => {
				// no fields
			},
			ShowDialog, 0x8A => {
				// TODO: dialog definition
			}
		},
		SERVER => {
			ConfirmTeleport, 0x00 => {
				teleport_id: VarInt
			},
			QueryBlockEntityTag, 0x01 => {
				transaction_id: VarInt,
				location: Position
			},
			BundleItemSelected, 0x02 => {
				// TODO
			},
			ChangeDifficultyServer, 0x03 => {
				new_difficulty: GameDifficulty
			},
			ChangeGameMode, 0x04 => {
				gamemode: u8
			},
			AcknowledgeMessage, 0x05 => {
				message_count: VarInt
			},
			ChatCommand, 0x06 => {
				command: String
			},
			SignedChatCommand, 0x07 => {
				// TODO: argument signatures array + acknowledged bitset
			},
			ChatMessage, 0x08 => {
				// TODO: signature + acknowledged bitset
			},
			PlayerSession, 0x09 => {
				// TODO: public key + signature
			},
			ChunkBatchReceived, 0x0A #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Batch_Received"] => {
				#[doc = "Chunks received per tick"]
				rate: f32
			},
			ClientCommand, 0x0B #[doc = "https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Command"] => {
				action: ClientStatusAction
			},
			ClientTickEnd, 0x0C => {
				// none
			},
			ClientInformationPlay, 0x0D => {
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
			CommandSuggestionsRequest, 0x0E => {
				transaction_id: VarInt,
				text: String
			},
			AcknowledgeConfiguration, 0x0F => {
				// no fields
			},
			ClickContainerButton, 0x10 => {
				window_id: VarInt,
				button_id: VarInt
			},
			ClickContainer, 0x11 => {
				// TODO: changed slots array + carried item
			},
			CloseContainerServer, 0x12 => {
				window_id: VarInt
			},
			ChangeContainerSlotState, 0x13 => {
				#[doc = "This is the ID of the slot that was changed."]
				slot_id: VarInt,
				#[doc = "This is the ID of the window that was changed."]
				window_id: VarInt,
				#[doc = "The new state of the slot. True for enabled, false for disabled."]
				state: bool
			},
			CookieResponsePlay, 0x14 => {
				key: String,
				payload: PrefixedOptional<PrefixedArray<u8>>
			},
			ServerboundPluginMessagePlay, 0x15 => {
				channel: String,
				data: Vec<u8>
			},
			DebugSubscriptionRequest, 0x16 => {
				request_type: VarInt
			},
			EditBook, 0x17 => {
				#[doc = "The hotbar slot where the written book is located"]
				slot: VarInt,
				#[doc = "Text from each page. Maximum string length is 1024 chars."]
				entries: PrefixedArray<String>,
				#[doc = "Title of book. Present if book is being signed, not present if book is being edited."]
				title: PrefixedOptional<String>
			},
			QueryEntityTag, 0x18 => {
				transaction_id: VarInt,
				entity_id: VarInt
			},
			Interact, 0x19 => {
				#[doc = "The ID of the entity to interact."]
				entity_id: VarInt,
				typ: InteractType,
				#[mc(deserialize_if = typ == InteractType::InteractAt)]
				target_x: Option<f32>,
				#[mc(deserialize_if = typ == InteractType::InteractAt)]
				target_y: Option<f32>,
				#[mc(deserialize_if = typ == InteractType::InteractAt)]
				target_z: Option<f32>,
				#[mc(deserialize_if = typ == InteractType::InteractAt)]
				hand: Option<InteractHand>,
				sneak_key_pressed: bool
			},
			JigsawGenerate, 0x1A => {
				location: Position,
				levels: VarInt,
				keep_jigsaws: bool
			},
			ServerboundKeepAlive, 0x1B => {
				keep_alive_id: i64
			},
			LockDifficulty, 0x1C => {
				locked: bool
			},
			SetPlayerPosition, 0x1D => { // TODO: position and movement validation -- check docs here?
				x: f64,
				#[doc = "Feet y position. Head - 1.62"]
				y: f64,
				z: f64,
				flags: PlayerPositionFlags
			},
			SetPlayerPositionRotation, 0x1E => {
				x: f64,
				#[doc = "Feet y position. Head - 1.62"]
				y: f64,
				z: f64,
				yaw: f32,
				pitch: f32,
				flags: PlayerPositionFlags
			},
			SetPlayerRotation, 0x1F => {
				yaw: f32,
				pitch: f32,
				flags: PlayerPositionFlags
			},
			SetPlayerMovementFlags, 0x20 => {
				flags: PlayerPositionFlags
			},
			MoveVehicleServer, 0x21 => {
				x: f64,
				y: f64,
				z: f64,
				yaw: f32,
				pitch: f32
			},
			PaddleBoat, 0x22 => {
				left_paddle_turning: bool,
				right_paddle_turning: bool
			},
			PickItemFromBlock, 0x23 => {
				location: Position,
				include_data: bool
			},
			PickItemFromEntity, 0x24 => {
				entity_id: VarInt,
				include_data: bool
			},
			PingRequestPlay, 0x25 => {
				payload: i64
			},
			PlaceRecipe, 0x26 => {
				// TODO: window id + recipe id + make all
			},
			PlayerAbilitiesServer, 0x27 => {
				flags: PlayerAbilityFlags
			},
			PlayerAction, 0x28 => {
				status: VarInt,
				location: Position,
				face: u8,
				sequence_id: VarInt
			},
			PlayerCommand, 0x29 => {
				entity_id: VarInt,
				#[doc = "0 to 100 for jumping on a horse"]
				jump_boost: VarInt // todo: FixedVarInt?
			},
			PlayerInput, 0x2A => {
				flags: PlayerInputFlags
			},
			PlayerLoaded, 0x2B => {
				// none
			},
			PongPlay, 0x2C => {
				id: i32
			},
			ChangeRecipeBookSettings, 0x2D => {
				book_id: VarInt,
				is_open: bool,
				is_filter_active: bool
			},
			SetSeenRecipe, 0x2E => {
				recipe_id: VarInt
			},
			RenameItem, 0x2F => {
				item_name: String
			},
			ResourcePackResponse, 0x30 => {
				uuid: Uuid,
				result: VarInt
			},
			SeenAdvancements, 0x31 => {
				// TODO: action + optional tab id
			},
			SelectTrade, 0x32 => {
				selected_slot: VarInt
			},
			SetBeaconEffect, 0x33 => {
				// TODO: optional primary/secondary effects
			},
			SetHeldItemServer, 0x34 => {
				slot: i16
			},
			ProgramCommandBlock, 0x35 => {
				// TODO
			},
			ProgramCommandBlockMinecart, 0x36 => {
				// TODO
			},
			SetCreativeModeSlot, 0x37 => {
				slot: i16,
				clicked_item: SlotData
			},
			ProgramJigsawBlock, 0x38 => {
				// TODO
			},
			ProgramStructureBlock, 0x39 => {
				// TODO
			},
			SetTestBlock, 0x3A => {
				// TODO
			},
			UpdateSign, 0x3B => {
				location: Position,
				is_front_text: bool,
				line_1: String,
				line_2: String,
				line_3: String,
				line_4: String
			},
			SwingArm, 0x3C => {
				hand: VarInt
			},
			TeleportToEntity, 0x3D => {
				target_player: Uuid
			},
			TestInstanceBlockAction, 0x3E => {
				// TODO
			},
			UseItemOn, 0x3F => {
				// TODO: hand + location + face + cursor + flags + sequence
			},
			UseItem, 0x40 => {
				// TODO: hand + sequence + rotation
			},
			CustomClickAction, 0x41 => {
				// TODO: identifier + NBT payload
			}
		}
	}
});
