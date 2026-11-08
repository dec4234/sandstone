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
use crate::protocol::game::effects::particle::Particle;
use crate::protocol::game::effects::sound::{SoundCategory, SoundEvent};
use crate::protocol::game::entity::EntityMetadata;
use crate::protocol::game::info::registry::{ChatType, RegistryDataPacketInternal};
use crate::protocol::game::info::stats::advancement::Advancement;
use crate::protocol::game::player::interface::dialog::Dialog;
use crate::protocol::game::player::inventory::slotdata::SlotData;
use crate::protocol::game::player::inventory::slots::{ChangedSlot, HashedSlot, InventoryOperationMode, RecipeDisplay};
use crate::protocol::game::player::player_action::PlayerInfoUpdateData;
use crate::protocol::game::player::{ClientStatusAction, RespawnKeptData};
use crate::protocol::game::world::chunk::{ChunkData, LightData};
use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::packets::packet_parts::auth::PublicKeyNetwork;
use crate::protocol::packets::packet_parts::block::{
	BlockFace, BlockParticleAlternative, CommandBlockFlag, CommandBlockMode, SpecialBlockRotation, StructureBlockAction, StructureBlockFlags, StructureBlockMirror, StructureBlockMode, TestBlockMode,
	TestInstanceBlockActionAction, TestInstanceStatus,
};
use crate::protocol::packets::packet_parts::debug::{CustomReportDetail, DebugSampleType, DebugSubscriptionEvent, DebugSubscriptionUpdate};
use crate::protocol::packets::packet_parts::entity::{EntityStatusEnum, MinecartMoveStep};
use crate::protocol::packets::packet_parts::item::{MapColorPatch, MapIcons, Trade};
use crate::protocol::packets::packet_parts::player::{PlayerActionStatus, SeenAdvancementsAction, TeleportFlags, UseItemHand, WaypointData, WaypointOperation};
use crate::protocol::packets::packet_parts::scoreboard::{ObjectiveNumberFormat, ObjectiveType, UpdateScoreFormat, UpdateTeamOptions};
use crate::protocol::packets::packet_parts::sound::StopSoundDetails;
use crate::protocol::packets::packet_parts::{ChatTypeNetwork, PlayerAbilityFlags, PlayerInputFlags, PlayerPositionFlags};
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::StateBasedDeserializer;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::{JsonTextComponent, PlayerChatFilter, PlayerChatSignature, TextComponent};
use crate::protocol_types::datatypes::command::Node;
use crate::protocol_types::datatypes::game_types::{ChunkSectionPosition, GameDifficulty, Position, SectionBlockEntry, SourcePosition, WorldEventType};
use crate::protocol_types::datatypes::internal_types::{Angle, Either, IDorX, LpVec3, Mapping, RgbColor, TripleDouble};
use crate::protocol_types::datatypes::nbt::{NbtCompound, NbtTag};
use crate::protocol_types::datatypes::var_types::{VarInt, VarLong};
use crate::util::java::bitset::BitSet;
use packet_parts::ProtocolPropertyElement;
use packet_parts::stats::StatisticAward;
use packet_parts::status::StatusResponseSpec;
use packet_parts::{
	AddResourcePackSpec, AttributeProperty, BossBarUpdateAction, ChunkBiomeData, CustomReportDetails, EquipmentList, GameEventType, InteractHand, InteractType, LoginCookieResponseSpec,
	LoginPluginSpec, PropertySet, RecipeBookEntry, ResourcePackEntry, ServerLink, StonecutterRecipe, Tag, TooltipMatch,
};
use uuid::Uuid;

pub mod packet_definer;
pub mod packet_parts;

// https://minecraft.wiki/w/Java_Edition_protocol
packets!(v1_21 => { // version name is for reference only, has no effect
	HANDSHAKING => {
		SERVER => { // Server is the destination
			Handshaking, 0x00 #[doc = "Used to switch server into a different connection state. Should be sent shortly after TCP connection is opened."] #[doc = ""] #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Handshake>"] => {
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
			StatusResponse, 0x00 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Status_Response>"] => {
				response: StatusResponseSpec
			},
			PingResponse, 0x01 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pong_Response_(status)>"] => {
				payload: u64
			}
		},
		SERVER => {
			StatusRequest, 0x00 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Status_Request>"] => {
				// none
			},
			PingRequest, 0x01 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping_Request_(status)>"] => {
				payload: i64
			}
		}
	},
	LOGIN => {
		CLIENT => {
			LoginDisconnect, 0x00 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disconnect_(login)>"] => {
				reason: JsonTextComponent
			},
			EncryptionRequest, 0x01 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Encryption_Request>"] => {
				server_id: String,
				public_key: PrefixedArray<u8>,
				verify_token: PrefixedArray<u8>,
				#[doc = "Set this to false if you want encryption but want to be in offline mode"]
				should_authenticate_with_mojang: bool
			},
			LoginSuccess, 0x02 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_Success>"] => { // todo: needs to be fixed
				uuid: Uuid,
				username: String,
				array: PrefixedArray<ProtocolPropertyElement>
			},
			SetCompression, 0x03 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Compression>"] => {
				#[doc = "The threshold for compression, in bytes. If the packet size is larger than this, it will be compressed."]
				threshold: VarInt
			},
			LoginPluginRequest, 0x04 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_Plugin_Request>"] => {
				message_id: VarInt,
				#[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Plugin_channels>"]
				channel: String,
				data: Vec<u8>
			},
			LoginCookieRequest, 0x05 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Request>"] => {
				key: String
			}
		},
		SERVER => {
			LoginStart, 0x00 #[doc = "Initiate the login procedure for a client."] #[doc = ""] #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_Start>"] => {
				username: String,
				uuid: Uuid
			},
			EncryptionResponse, 0x01 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Encryption_Response>"] => {
				shared_secret: PrefixedArray<u8>,
				verify_token: PrefixedArray<u8>
			},
			LoginPluginResponse, 0x02 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_Plugin_Response>"] => {
				response: LoginPluginSpec
			},
			LoginAcknowledged, 0x03 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_Acknowledged>"] => {
				// none
			},
			LoginCookieResponse, 0x04 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Response>"] => {
				spec: LoginCookieResponseSpec
			}
		}
	},
	CONFIGURATION => {
		CLIENT => {
			ConfigCookieRequest, 0x00 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Request>"] => {
				key: String
			},
			ClientboundPluginMessage, 0x01 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(clientbound)>"] => {
				#[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Plugin_channels>"]
				channel: String,
				data: Vec<u8>
			},
			ConfigDisconnect, 0x02 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disconnect>"] => {
				reason: TextComponent
			},
			FinishConfiguration, 0x03 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Finish_Configuration>"] => {
				// none
			},
			KeepAlive, 0x04 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(clientbound)>"] => {
				keep_alive_id: i64
			},
			ConfigurationPing, 0x05 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping>"] => {
				payload: i32
			},
			ResetChat, 0x06 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Reset_Chat>"] => {
				// none
			},
			RegistryData, 0x07 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Registry_Data>"] => {
				packet: RegistryDataPacketInternal
			},
			RemoveResourcePack, 0x08 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Remove_Resource_Pack>"] => {
				uuid: PrefixedOptional<Uuid>
			},
			AddResourcePack, 0x09 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Add_Resource_Pack>"] => {
				spec: AddResourcePackSpec
			},
			ConfigStoreCookie, 0x0A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Store_Cookie>"] => {
				key: String,
				payload: PrefixedArray<u8>
			},
			ConfigTransfer, 0x0B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Transfer>"] => {
				host: String,
				port: VarInt
			},
			FeatureFlags, 0x0C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Feature_Flags>"] => {
				total: VarInt,
				flags: Vec<String>
			},
			UpdateTags, 0x0D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Tags>"] => {
				tags: PrefixedArray<Mapping<PrefixedArray<Tag>>>
			},
			ClientboundKnownPacks, 0x0E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Known_Packs_(clientbound)>"] => {
				entries: PrefixedArray<ResourcePackEntry>
			},
			ConfigCustomReportDetails, 0x0F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Report_Details>"] => {
				details: PrefixedArray<CustomReportDetails>
			},
			ConfigServerLinks, 0x10 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Server_Links>"] => {
				links: PrefixedArray<ServerLink>
			}
		},
		SERVER => {
			ClientInformation, 0x00 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Information>"] => {
				locale: String,
				view_distance: i8,
				#[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Chat#Client_chat_mode>"]
				chat_mode: VarInt,
				chat_colors: bool,
				displayed_skin_parts: u8,
				main_hand: VarInt,
				enable_text_filtering: bool,
				allow_server_listing: bool,
				particle_status: VarInt
			},
			CookieResponse, 0x01 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Response>"] => {
				key: String,
				payload: PrefixedOptional<PrefixedArray<u8>>
			},
			ServerboundPluginMessage, 0x02 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(serverbound)>"] => {
				#[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Plugin_channels>"]
				channel: String,
				data: Vec<u8>
			},
			AcknowledgeFinishConfiguration, 0x03 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Acknowledge_Finish_Configuration>"] => {
				// none
			},
			ServerboundKeepAliveConfig, 0x04 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(serverbound)>"] => {
				keep_alive_id: i64
			},
			ConfigPong, 0x05 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pong>"] => {
				id: i32
			},
			ConfigResourcePackResponse, 0x06 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Resource_Pack_Response>"] => {
				uuid: Uuid,
				result: VarInt
			},
			ServerboundKnownPacks, 0x07 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Known_Packs_(serverbound)>"] => {
				entries: PrefixedArray<ResourcePackEntry>
			},
			ConfigCustomClickAction, 0x08 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Click_Action>"] => {
				id: String,
				payload: NbtCompound
			}
		}
	},
	PLAY => {
		CLIENT => {
			BundleDelimiter, 0x00 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Bundle_Delimiter>"] => {
				// no fields
			},
			SpawnEntity, 0x01 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Spawn_Entity>"] => {
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
				#[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Object_data>"]
				data: VarInt
			},
			EntityAnimation, 0x02 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Entity_Animation>"] => {
				entity_id: VarInt,
				animation: u8
			},
			AwardStatistics, 0x03 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Award_Statistics>"] => {
				stats: PrefixedArray<StatisticAward>
			},
			AcknowledgeBlockChange, 0x04 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Acknowledge_Block_Change>"] => {
				sequence_id: VarInt
			},
			SetBlockDestroyStage, 0x05 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Block_Destroy_Stage>"] => {
				entity_id: VarInt,
				location: Position,
				#[doc = "0-9 to set, any other value removes the stage"]
				destroy_stage: i8
			},
			BlockEntityData, 0x06 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Block_Entity_Data>"] => {
				location: Position,
				typ: VarInt,
				data: NbtTag
			},
			BlockAction, 0x07 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Block_Action>"] => {
				location: Position,
				#[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Block_actions>"]
				action_id: u8,
				action_param: u8,
				block_type: VarInt
			},
			BlockUpdate, 0x08 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Block_Update>"] => {
				location: Position,
				block_id: VarInt
			},
			BossBar, 0x09 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Boss_Bar>"] => {
				uuid: Uuid,
				action: BossBarUpdateAction
			},
			ChangeDifficulty, 0x0A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Difficulty>"] => {
				difficulty: GameDifficulty,
				difficulty_locked: bool
			},
			ChunkBatchFinished, 0x0B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Batch_Finished>"] => {
				size: VarInt
			},
			ChunkBatchStart, 0x0C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Batch_Start>"] => {
				// no fields
			},
			ChunkBiomes, 0x0D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Biomes>"] => {
				chunk_biome_data: PrefixedArray<ChunkBiomeData>
			},
			ClearTitles, 0x0E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Clear_Titles>"] => {
				reset: bool
			},
			CommandSuggestionsResponse, 0x0F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Command_Suggestions_Response>"] => {
				id: VarInt,
				start: VarInt,
				length: VarInt,
				matches: PrefixedArray<TooltipMatch>
			},
			CommandsGraph, 0x10 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Commands>"] => {
				nodes: PrefixedArray<Node>,
				root_index: VarInt
			},
			CloseContainer, 0x11 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Close_Container>"] => {
				window_id: VarInt
			},
			SetContainerContent, 0x12 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Container_Content>"] => {
				window_id: VarInt,
				state_id: VarInt,
				slot_data: PrefixedArray<SlotData>,
				carried_item: SlotData
			},
			SetContainerProperty, 0x13 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Container_Property>"] => {
				window_id: VarInt,
				property: i16,
				value: i16
			},
			SetContainerSlot, 0x14 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Container_Slot>"] => {
				window_id: VarInt,
				state_id: VarInt,
				slot: i16,
				slot_data: SlotData
			},
			CookieRequestPlay, 0x15 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Request>"] => {
				key: String
			},
			SetCooldown, 0x16 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Cooldown>"] => {
				cooldown_group: String,
				cooldown_ticks: VarInt
			},
			ChatSuggestions, 0x17 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chat_Suggestions>"] => {
				action: VarInt,
				entries: PrefixedArray<String>
			},
			ClientboundPluginMessagePlay, 0x18 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(clientbound)>"] => {
				#[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Plugin_channels>"]
				channel: String,
				data: Vec<u8>
			},
			DamageEvent, 0x19 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Damage_Event>"] => {
				entity_id: VarInt,
				source_type_id: VarInt,
				source_cause_id: VarInt,
				source_direct_id: VarInt,
				source_position: PrefixedOptional<SourcePosition>
			},
			DebugBlockValue, 0x1A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Block_Value>"] => {
				location: Position,
				update: DebugSubscriptionUpdate
			},
			DebugChunkValue, 0x1B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Chunk_Value>"] => {
				z: i32,
				x: i32,
				update: DebugSubscriptionUpdate
			},
			DebugEntityValue, 0x1C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Entity_Value>"] => {
				entity_id: VarInt,
				update: DebugSubscriptionUpdate
			},
			DebugEvent, 0x1D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Event>"] => {
				event: DebugSubscriptionEvent
			},
			DebugSample, 0x1E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Sample>"] => {
				#[doc = "Array of type-dependent samples."]
				sample: PrefixedArray<i64>,
				sample_type: DebugSampleType
			},
			DeleteMessage, 0x1F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Delete_Message>"] => {
				message_id: VarInt,
				#[mc(deserialize_if = message_id.0 == 0)]
				signature: Option<[u8; 256]>
			},
			DisconnectPlay, 0x20 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disconnect>"] => {
				reason: TextComponent
			},
			DisguisedChatMessage, 0x21 #[doc = "Send client a chat message without any signing information"] #[doc = ""] #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Disguised_Chat_Message>"] => {
				message: TextComponent,
				chat_type: IDorX<Box<ChatTypeNetwork>>,
				sender_name: TextComponent,
				target_name: PrefixedOptional<TextComponent>
			},
			EntityEvent, 0x22 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Entity_statuses>"] #[doc = ""] #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Entity_Event>"] => {
				entity_id: i32,
				entity_status: EntityStatusEnum
			},
			TeleportEntity, 0x23 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Teleport_Entity>"] => {
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
			Explosion, 0x24 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Explosion>"] => {
				x: f64,
				y: f64,
				z: f64,
				radius: f32,
				block_count: i32,
				player_delta_velocity: PrefixedOptional<TripleDouble>,
				explosion_particle_id: VarInt,
				explosion_particle_data: Particle,
				explosion_sound: IDorX<SoundEvent>,
				block_particle_alternatives: BlockParticleAlternative
			},
			UnloadChunk, 0x25 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Unload_Chunk>"] => {
				#[doc = "Note: Chunk Z is sent before Chunk X"]
				z: i32,
				x: i32
			},
			GameEvent, 0x26 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Game_Event>"] => {
				event: GameEventType,
				value: f32
			},
			GameTestHighlightPosition, 0x27 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Game_Test_Highlight_Position>"] => {
				absolute_location: Position,
				relative_location: Position
			},
			OpenHorseScreen, 0x28 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Open_Horse_Screen>"] => {
				window_id: VarInt,
				slot_count: VarInt,
				entity_id: i32
			},
			HurtAnimation, 0x29 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Hurt_Animation>"] => {
				entity_id: VarInt,
				yaw: f32
			},
			InitializeWorldBorder, 0x2A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Initialize_World_Border>"] => {
				x: f64,
				z: f64,
				old_diameter: f64,
				new_diameter: f64,
				speed: VarLong,
				portal_teleport_boundary: VarInt,
				warning_blocks: VarInt,
				warning_time: VarInt
			},
			ClientboundKeepAlive, 0x2B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(clientbound)>"] => {
				keep_alive_id: i64
			},
			ChunkDataUpdateLight, 0x2C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Data_and_Update_Light>"] => {
				x: i32,
				z: i32,
				#[doc = "Chunk byte data"]
				data: ChunkData,
				light: LightData
			},
			WorldEvent, 0x2D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#World_Event>"] => {
				event: WorldEventType,
				location: Position,
				data: i32,
				disable_relative_volume: bool
			},
			ParticlePacket, 0x2E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Particle>"] => {
				long_distance: bool,
				always_visibile: bool,
				x: f64,
				y: f64,
				z: f64,
				offset_x: f32,
				offset_y: f32,
				offset_z: f32,
				max_speed: f32,
				particle_count: i32,
				particle_id: VarInt,
				data: Particle
			},
			UpdateLight, 0x2F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Light>"] => {
				x_chunk: VarInt,
				y_chunk: VarInt,
				data: LightData
			},
			LoginInfo, 0x30 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Login_(play)>"] => {
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
			MapData, 0x31 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Map_Data>"] => {
				map_id: VarInt,
				scale: i8,
				locked: bool,
				icons: MapIcons,
				color_patch: MapColorPatch
			},
			MerchantOffers, 0x32 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Merchant_Offers>"] => {
				window_id: VarInt,
				trades: PrefixedArray<Trade>,
				villager_level: VarInt,
				experience: VarInt,
				is_regular_villager: bool,
				can_restock: bool
			},
			UpdateEntityPosition, 0x33 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Entity_Position>"] => {
				entity_id: VarInt,
				#[doc = "Change in X position as currentX * 4096 - prevX * 4096"]
				x_delta: i16,
				#[doc = "Change in Y position as currentY * 4096 - prevY * 4096"]
				y_delta: i16,
				#[doc = "Change in Z position as currentZ * 4096 - prevZ * 4096"]
				z_delta: i16,
				on_ground: bool
			},
			UpdateEntityPostitionRotation, 0x34 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Entity_Position_and_Rotation>"] => {
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
			MoveMinecartAlongTrack, 0x35 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Move_Minecart_Along_Track>"] => {
				entity_id: VarInt,
				steps: PrefixedArray<MinecartMoveStep>
			},
			UpdateEntityRotation, 0x36 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Entity_Rotation>"] => {
				entity_id: VarInt,
				yaw: Angle,
				pitch: Angle,
				on_ground: bool
			},
			MoveVehicle, 0x37 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Move_Vehicle_(clientbound)>"] => {
				x: f64,
				y: f64,
				z: f64,
				yaw: f32,
				pitch: f32
			},
			OpenBook, 0x38 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Open_Book>"] => {
				hand: VarInt
			},
			OpenScreen, 0x39 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Open_Screen>"] => {
				window_id: VarInt,
				#[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Inventory>"]
				window_type: VarInt,
				window_title: TextComponent
			},
			OpenSignEditor, 0x3A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Open_Sign_Editor>"] => {
				location: Position,
				is_front_text: bool
			},
			PingPlay, 0x3B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Ping>"] => {
				id: i32
			},
			PingResponsePlay, 0x3C => {
				payload: i64
			},
			PlaceGhostRecipe, 0x3D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Place_Ghost_Recipe>"] => {
				window_id: VarInt,
				recipe_display: RecipeDisplay
			},
			PlayerAbilities, 0x3E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Abilities_(clientbound)>"] => {
				flags: PlayerAbilityFlags,
				flying_speed: f32,
				fov_modifier: f32
			},
			PlayerChatMessage, 0x3F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Chat_Message>"] => {
				global_index: VarInt,
				sender: Uuid,
				index: VarInt,
				message_signature_bytes: PrefixedOptional<[u8; 256]>,
				message: String,
				timestamp: i64,
				salt: i64,
				signatures: Box<PrefixedArray<PlayerChatSignature>>,
				unsigned_content: PrefixedOptional<Box<TextComponent>>,
				filter: PlayerChatFilter,
				#[doc = "Only present if the Filter Type is Partially Filtered. Specifies the indices at which characters in the original message string should be replaced with the # symbol (i.e., filtered) by the vanilla client"]
				#[mc(deserialize_if = filter == PlayerChatFilter::PartiallyFiltered)]
				filter_type_bits: Option<BitSet>,
				#[doc = "Either the type of chat in the minecraft:chat_type registry, defined by the Registry Data packet, or an inline definition."]
				chat_type: IDorX<ChatType>,
				#[doc = "The name of the one sending the message, usually the sender's display name."]
				sender_name: Box<TextComponent>,
				#[doc = "The name of the one receiving the message, usually the receiver's display name."]
				target_name: PrefixedOptional<Box<TextComponent>>
			},
			EndCombat, 0x40 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#End_Combat>"] => {
				duration: VarInt
			},
			EnterCombat, 0x41 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Enter_Combat>"] => {
				// no fields
			},
			CombatDeath, 0x42 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Combat_Death>"] => {
				#[doc = "Entity ID of the player that died (should match the client's entity ID)"]
				id: VarInt,
				message: TextComponent
			},
			PlayerInfoRemove, 0x43 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Info_Remove>"] => {
				players: PrefixedArray<Uuid>
			},
			PlayerInfoUpdate, 0x44 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Info_Update>"] => {
				data: PlayerInfoUpdateData
			},
			LookAt, 0x45 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Look_At>"] => {
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
			SyncPlayerPosition, 0x46 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Synchronize_Player_Position>"] => {
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
				flags: TeleportFlags
			},
			PlayerRotation, 0x47 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Rotation>"] => {
				#[doc = "Rotation on the X axis, in degrees."]
				yaw: f32,
				relative_yaw: bool,
				#[doc = "Rotation on the Y axis, in degrees."]
				pitch: f32,
				relative_pitch: bool
			},
			RecipeBookAdd, 0x48 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Recipe_Book_Add>"] => {
				recipes: PrefixedArray<RecipeBookEntry>,
				replace: bool
			},
			RecipeBookRemove, 0x49 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Recipe_Book_Remove>"] => {
				recipes: PrefixedArray<VarInt>
			},
			RecipeBookSettings, 0x4A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Recipe_Book_Settings>"] => {
				crafting_open: bool,
				crafting_filter: bool,
				smelting_open: bool,
				smelting_filter: bool,
				blasting_open: bool,
				blasting_filter: bool,
				smoking_open: bool,
				smoking_filter: bool
			},
			RemoveEntities, 0x4B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Remove_Entities>"] => {
				#[doc = "A prefixed array of entity ids to be destroyed"]
				entities: PrefixedArray<VarInt>
			},
			RemoveEntityEffect, 0x4C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Remove_Entity_Effect>"] => {
				entity_id: VarInt,
				effect_id: VarInt
			},
			ResetScore, 0x4D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Reset_Score>"] => {
				entity_name: String,
				objective_name: PrefixedOptional<String>
			},
			RemoveResourcePackPlay, 0x4E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Remove_Resource_Pack>"] => {
				uuid: PrefixedOptional<Uuid>
			},
			AddResourcePackPlay, 0x4F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Add_Resource_Pack>"] => {
				spec: AddResourcePackSpec
			},
			Respawn, 0x50 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Respawn>"] => {
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
			SetHeadRotation, 0x51 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Head_Rotation>"] => {
				entity_id: VarInt,
				head_yaw: Angle
			},
			SectionBlocksUpdate, 0x52 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Section_Blocks>"] => {
				chunk_section_position: ChunkSectionPosition,
				blocks: PrefixedArray<SectionBlockEntry>
			},
			SelectAdvancementsTab, 0x53 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Select_Advancements_Tab>"] => {
				tab_id: PrefixedOptional<String>
			},
			ServerData, 0x54 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Server_Data>"] => {
				motd: TextComponent,
				icon: PrefixedOptional<PrefixedArray<u8>>
			},
			SetActionBarText, 0x55 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Action_Bar_Text>"] => {
				action_bar_text: TextComponent
			},
			SetBorderCenter, 0x56 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Border_Center>"] => {
				x: f64,
				z: f64
			},
			SetBorderLerpSize, 0x57 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Border_Lerp_Size>"] => {
				old_diameter: f64,
				new_diameter: f64,
				speed: VarLong
			},
			SetBorderSize, 0x58 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Border_Size>"] => {
				diameter: f64
			},
			SetBorderWarningDelay, 0x59 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Border_Warning_Delay>"] => {
				warning_delay: VarInt
			},
			SetBorderWarningDistance, 0x5A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Border_Warning_Distance>"] => {
				warning_distance: VarInt
			},
			SetCamera, 0x5B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Camera>"] => {
				camera_entity_id: VarInt
			},
			SetCenterChunk, 0x5C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Center_Chunk>"] => {
				x: VarInt,
				z: VarInt
			},
			SetRenderDistance, 0x5D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Render_Distance>"] => {
				render_distance: VarInt
			},
			SetCursorItem, 0x5E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Cursor_Item>"] => {
				item: SlotData
			},
			SetDefaultSpawnPosition, 0x5F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Default_Spawn_Position>"] => {
				dimension_name: String,
				location: Position,
				yaw: f32,
				pitch: f32
			},
			DisplayObjective, 0x60 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Display_Objective>"] => {
				position: VarInt,
				score_name: String
			},
			SetEntityMetadata, 0x61 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Entity_Metadata>"] => {
				entity_id: VarInt,
				metadata: EntityMetadata
			},
			LinkEntries, 0x62 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Link_Entities>"] => {
				attached_id: i32,
				holding_id: i32
			},
			SetEntityVelocity, 0x63 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Entity_Velocity>"] => {
				entity_id: VarInt,
				velocity: LpVec3
			},
			SetEquipment, 0x64 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Equipment>"] => {
				entity_id: VarInt,
				equipment: EquipmentList
			},
			SetExperience, 0x65 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Experience>"] => {
				experience_bar: f32,
				level: VarInt,
				total_experience: VarInt
			},
			SetHealth, 0x66 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Health>"] => {
				health: f32,
				food: VarInt,
				saturation: f32
			},
			SetHeldItem, 0x67 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Held_Item_(clientbound)>"] => {
				slot: VarInt
			},
			UpdateObjectives, 0x68 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Objectives>"] => {
				objective_name: String,
				mode: i8,
				#[mc(deserialize_if = mode == 0 || mode == 2)]
				objective_value: Option<TextComponent>,
				#[mc(deserialize_if = mode == 0 || mode == 2)]
				typ: Option<ObjectiveType>,
				#[mc(deserialize_if = mode == 0 || mode == 2)]
				has_number_format: Option<bool>,
				#[mc(deserialize_if = (mode == 0 || mode == 2) && has_number_format.unwrap())]
				number_format: Option<ObjectiveNumberFormat>
			},
			SetPassengers, 0x69 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Passengers>"] => {
				entity_id: VarInt,
				passengers: PrefixedArray<VarInt>
			},
			SetPlayerInventorySlot, 0x6A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Player_Inventory_Slot>"] => {
				slot: VarInt,
				item: SlotData
			},
			UpdateTeams, 0x6B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Teams>"] => {
				team_name: String,
				team_details: UpdateTeamOptions
			},
			UpdateScore, 0x6C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Score>"] => {
				entity_name: String,
				objective_name: String,
				value: VarInt,
				display_name: PrefixedOptional<TextComponent>,
				number_format: PrefixedOptional<UpdateScoreFormat>
			},
			SetSimulationDistance, 0x6D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Simulation_Distance>"] => {
				simulation_distance: VarInt
			},
			SetSubtitleText, 0x6E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Subtitle_Text>"] => {
				subtitle_text: TextComponent
			},
			UpdateTime, 0x6F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Time>"] => {
				world_age: i64,
				time_of_day: i64,
				time_of_day_increasing: bool
			},
			SetTitleText, 0x70 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Title_Text>"] => {
				title_text: TextComponent
			},
			SetTitleAnimationTimes, 0x71 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Title_Animation_Times>"] => {
				fade_in: i32,
				stay: i32,
				fade_out: i32
			},
			EntitySoundEffect, 0x72 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Entity_Sound_Effect>"] => {
				sound_event: IDorX<SoundEvent>,
				sound_category: SoundCategory,
				entity_id: VarInt,
				volume: f32,
				pitch: f32,
				seed: i64
			},
			SoundEffect, 0x73 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Sound_Effect>"] => {
				sound_event: IDorX<SoundEvent>,
				sound_category: SoundCategory,
				entity_id: VarInt,
				volume: f32,
				pitch: f32,
				seed: i64
			},
			StartConfiguration, 0x74 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Start_Configuration>"] => {
				// no fields
			},
			StopSound, 0x75 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Stop_Sound>"] => {
				details: StopSoundDetails
			},
			StoreCookie, 0x76 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Store_Cookie>"] => {
				key: String,
				payload: PrefixedArray<u8>
			},
			SystemChatMessage, 0x77 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#System_Chat_Message>"] => {
				content: TextComponent,
				overlay: bool
			},
			SetTabListHeaderAndFooter, 0x78 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Tab_List_Header_And_Footer>"] => {
				header: TextComponent,
				footer: TextComponent
			},
			TagQueryResponse, 0x79 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Tag_Query_Response>"] => {
				transaction_id: VarInt,
				nbt: NbtTag
			},
			PickupItem, 0x7A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pickup_Item>"] => {
				collected_entity_id: VarInt,
				collector_entity_id: VarInt,
				pickup_item_count: VarInt
			},
			SynchronizeVehiclePosition, 0x7B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Synchronize_Vehicle_Position>"] => {
				entity_id: VarInt,
				x: f64,
				y: f64,
				z: f64,
				velocity_x: f64,
				velocity_y: f64,
				velocity_z: f64,
				yaw: f32,
				pitch: f32,
				flags: TeleportFlags,
				on_ground: bool
			},
			TestInstanceBlockStatus, 0x7C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Test_Instance_Block_Status>"] => {
				status: TextComponent,
				has_size: bool,
				#[mc(deserialize_if = has_size)]
				size_x: Option<f64>,
				#[mc(deserialize_if = has_size)]
				size_y: Option<f64>,
				#[mc(deserialize_if = has_size)]
				size_z: Option<f64>
			},
			SetTickingState, 0x7D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Ticking_State>"] => {
				tick_rate: f32,
				is_frozen: bool
			},
			StepTick, 0x7E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Step_Tick>"] => {
				tick_steps: VarInt
			},
			Transfer, 0x7F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Transfer>"] => {
				host: String,
				port: VarInt
			},
			UpdateAdvancements, 0x80 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Advancements>"] => {
				reset: bool,
				advancement_mapping: PrefixedArray<Mapping<Advancement>>,
				identifiers: PrefixedArray<String>,
				progress_mapping: PrefixedArray<Mapping<PrefixedArray<Mapping<PrefixedOptional<i64>>>>>,
				show_advancements: bool
			},
			UpdateAttributes, 0x81 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Attributes>"] => {
				entity_id: VarInt,
				modifiers: PrefixedArray<AttributeProperty>
			},
			EntityEffect, 0x82 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Entity_Effect>"] => {
				entity_id: VarInt,
				effect_id: VarInt,
				amplifier: i8,
				duration: VarInt,
				flags: i8
			},
			UpdateRecipes, 0x83 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Recipes>"] => {
				property_sets: PrefixedArray<PropertySet>,
				stonecutter_recipes: PrefixedArray<StonecutterRecipe>
			},
			UpdateTagsPlay, 0x84 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Tags>"] => {
				tags: PrefixedArray<Mapping<PrefixedArray<Tag>>>
			},
			ProjectilePower, 0x85 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Projectile_Power>"] => {
				entity_id: VarInt,
				power: f64
			},
			CustomReportDetails, 0x86 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Report_Details>"] => {
				details: PrefixedArray<CustomReportDetail>
			},
			ServerLinks, 0x87 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Server_Links>"] => {
				links: PrefixedArray<ServerLink>
			},
			Waypoint, 0x88 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Waypoint>"] => {
				operation: WaypointOperation,
				identifier: Either<Uuid, String>,
				icon_style: String,
				color: PrefixedOptional<RgbColor>,
				waypoint_type: WaypointData
			},
			ClearDialog, 0x89 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Clear_Dialog>"] => {
				// no fields
			},
			ShowDialog, 0x8A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Show_Dialog_(play)>"] => {
				dialog: Box<Dialog>
			}
		},
		SERVER => {
			ConfirmTeleport, 0x00 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Confirm_Teleportation>"] => {
				teleport_id: VarInt
			},
			QueryBlockEntityTag, 0x01 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Query_Block_Entity_Tag>"] => {
				transaction_id: VarInt,
				location: Position
			},
			BundleItemSelected, 0x02 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Bundle_Item_Selected>"] => {
				slot_of_bundle: VarInt,
				slot_in_bundle: VarInt
			},
			ChangeDifficultyServer, 0x03 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Difficulty_2>"] => {
				new_difficulty: GameDifficulty
			},
			ChangeGameMode, 0x04 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Game_Mode>"] => {
				gamemode: u8
			},
			AcknowledgeMessage, 0x05 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Acknowledge_Message>"] => {
				message_count: VarInt
			},
			ChatCommand, 0x06 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chat_Command>"] => {
				command: String
			},
			SignedChatCommand, 0x07 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Signed_Chat_Command>"] => {
				#[doc = "The command typed by the client excluding the /"]
				command: String,
				time: i64,
				salt: i64,
				argument_signatures: PrefixedArray<Mapping<Box<[u8; 256]>>>,
				message_count: VarInt,
				acknowledged: BitSet,
				checksum: i8
			},
			ChatMessage, 0x08 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chat_Message>"] => {
				message: String,
				time: i64,
				salt: i64,
				signature: PrefixedOptional<Box<[u8; 256]>>,
				message_count: VarInt,
				acknowledged: BitSet,
				checksum: i8
			},
			PlayerSession, 0x09 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Session>"] => {
				session_id: Uuid,
				public_key: Box<PublicKeyNetwork>
			},
			ChunkBatchReceived, 0x0A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Batch_Received>"] => {
				#[doc = "Chunks received per tick"]
				rate: f32
			},
			ClientCommand, 0x0B #[doc ="<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Status>"] => {
				action: ClientStatusAction
			},
			ClientTickEnd, 0x0C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Tick_End>"] => {
				// none
			},
			ClientInformationPlay, 0x0D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Client_Information>"] => {
				locale: String,
				view_distance: i8,
				#[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Chat#Client_chat_mode>"]
				chat_mode: VarInt,
				chat_colors: bool,
				displayed_skin_parts: u8,
				main_hand: VarInt,
				enable_text_filtering: bool,
				allow_server_listing: bool,
				particle_status: VarInt
			},
			CommandSuggestionsRequest, 0x0E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Command_Suggestions_Request>"] => {
				transaction_id: VarInt,
				text: String
			},
			AcknowledgeConfiguration, 0x0F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Acknowledge_Configuration>"] => {
				// no fields
			},
			ClickContainerButton, 0x10 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Click_Container_Button>"] => {
				window_id: VarInt,
				button_id: VarInt
			},
			ClickContainer, 0x11 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Click_Container>"] => {
				window_id: VarInt,
				#[doc = "The last received State ID from either a **Set Container Slot** or a **Set Container Content** packet."]
				state_id: VarInt,
				slot: i16,
				button: i8,
				mode: InventoryOperationMode,
				changed_slots: PrefixedArray<ChangedSlot>,
				#[doc = "Item carried by the cursor"]
				carried_item: HashedSlot
			},
			CloseContainerServer, 0x12 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Close_Container_2>"] => {
				window_id: VarInt
			},
			ChangeContainerSlotState, 0x13 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Container_Slot_State>"] => {
				#[doc = "This is the ID of the slot that was changed."]
				slot_id: VarInt,
				#[doc = "This is the ID of the window that was changed."]
				window_id: VarInt,
				#[doc = "The new state of the slot. True for enabled, false for disabled."]
				state: bool
			},
			CookieResponsePlay, 0x14 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Cookie_Response>"] => {
				key: String,
				payload: PrefixedOptional<PrefixedArray<u8>>
			},
			ServerboundPluginMessagePlay, 0x15 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Plugin_Message_(serverbound)>"] => {
				#[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Plugin_channels>"]
				channel: String,
				data: Vec<u8>
			},
			DebugSubscriptionRequest, 0x16 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Debug_Subscription_Request>"] => {
				request_type: VarInt
			},
			EditBook, 0x17 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Edit_Book>"] => {
				#[doc = "The hotbar slot where the written book is located"]
				slot: VarInt,
				#[doc = "Text from each page. Maximum string length is 1024 chars."]
				entries: PrefixedArray<String>,
				#[doc = "Title of book. Present if book is being signed, not present if book is being edited."]
				title: PrefixedOptional<String>
			},
			QueryEntityTag, 0x18 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Query_Entity_Tag>"] => {
				transaction_id: VarInt,
				entity_id: VarInt
			},
			Interact, 0x19 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Interact>"] => {
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
			JigsawGenerate, 0x1A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Jigsaw_Generate>"] => {
				location: Position,
				levels: VarInt,
				keep_jigsaws: bool
			},
			ServerboundKeepAlive, 0x1B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Keep_Alive_(serverbound)>"] => {
				keep_alive_id: i64
			},
			LockDifficulty, 0x1C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Lock_Difficulty>"] => {
				locked: bool
			},
			SetPlayerPosition, 0x1D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Player_Position>"] => { // TODO: position and movement validation -- check docs here?
				x: f64,
				#[doc = "Feet y position. Head - 1.62"]
				y: f64,
				z: f64,
				flags: PlayerPositionFlags
			},
			SetPlayerPositionRotation, 0x1E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Player_Position_and_Rotation>"] => {
				x: f64,
				#[doc = "Feet y position. Head - 1.62"]
				y: f64,
				z: f64,
				yaw: f32,
				pitch: f32,
				flags: PlayerPositionFlags
			},
			SetPlayerRotation, 0x1F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Player_Rotation>"] => {
				yaw: f32,
				pitch: f32,
				flags: PlayerPositionFlags
			},
			SetPlayerMovementFlags, 0x20 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Player_Movement_Flags>"] => {
				flags: PlayerPositionFlags
			},
			MoveVehicleServer, 0x21 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Move_Vehicle_(serverbound)>"] => {
				x: f64,
				y: f64,
				z: f64,
				yaw: f32,
				pitch: f32
			},
			PaddleBoat, 0x22 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Paddle_Boat>"] => {
				left_paddle_turning: bool,
				right_paddle_turning: bool
			},
			PickItemFromBlock, 0x23 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pick_Item_From_Block>"] => {
				location: Position,
				include_data: bool
			},
			PickItemFromEntity, 0x24 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pick_Item_From_Entity>"] => {
				entity_id: VarInt,
				include_data: bool
			},
			PingRequestPlay, 0x25 => {
				payload: i64
			},
			PlaceRecipe, 0x26 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Place_Recipe>"] => {
				window_id: VarInt,
				#[doc = "ID of recipe previously defined in **Recipe Book Add**."]
				recipe_id: VarInt,
				make_all: bool
			},
			PlayerAbilitiesServer, 0x27 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Abilities_(serverbound)>"] => {
				flags: PlayerAbilityFlags
			},
			PlayerAction, 0x28 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Action>"] => {
				status: PlayerActionStatus,
				location: Position,
				face: BlockFace,
				sequence_id: VarInt
			},
			PlayerCommand, 0x29 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Command>"] => {
				entity_id: VarInt,
				#[doc = "0 to 100 for jumping on a horse"]
				jump_boost: VarInt // todo: FixedVarInt?
			},
			PlayerInput, 0x2A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Input>"] => {
				flags: PlayerInputFlags
			},
			PlayerLoaded, 0x2B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Player_Loaded>"] => {
				// none
			},
			PongPlay, 0x2C #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Pong>"] => {
				id: i32
			},
			ChangeRecipeBookSettings, 0x2D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Change_Recipe_Book_Settings>"] => {
				book_id: VarInt,
				is_open: bool,
				is_filter_active: bool
			},
			SetSeenRecipe, 0x2E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Seen_Recipe>"] => {
				recipe_id: VarInt
			},
			RenameItem, 0x2F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Rename_Item>"] => {
				item_name: String
			},
			ResourcePackResponse, 0x30 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Resource_Pack_Response>"] => {
				uuid: Uuid,
				result: VarInt
			},
			SeenAdvancements, 0x31 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Seen_Advancements>"] => {
				action: SeenAdvancementsAction,
				#[mc(deserialize_if = action == SeenAdvancementsAction::OpenedTab)]
				tab_id: Option<String>
			},
			SelectTrade, 0x32 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Select_Trade>"] => {
				selected_slot: VarInt
			},
			SetBeaconEffect, 0x33 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Beacon_Effect>"] => {
				#[doc = "A potion ID"]
				primary_effect: PrefixedOptional<VarInt>,
				#[doc = "A potion ID"]
				secondary_effect: PrefixedOptional<VarInt>
			},
			SetHeldItemServer, 0x34 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Held_Item_(serverbound)>"] => {
				slot: i16
			},
			ProgramCommandBlock, 0x35 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Command_Block>"] => {
				location: Position,
				command: String,
				mode: CommandBlockMode,
				flags: CommandBlockFlag
			},
			ProgramCommandBlockMinecart, 0x36 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Command_Block_Minecart>"] => {
				entity_id: VarInt,
				command: String,
				track_output: bool
			},
			SetCreativeModeSlot, 0x37 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Creative_Mode_Slot>"] => {
				slot: i16,
				clicked_item: SlotData
			},
			ProgramJigsawBlock, 0x38 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Jigsaw_Block>"] => {
				location: Position,
				name: String,
				target: String,
				pool: String,
				final_state: String,
				joint_type: String,
				selection_priority: VarInt,
				placement_priority: VarInt
			},
			ProgramStructureBlock, 0x39 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Program_Structure_Block>"] => {
				location: Position,
				action: StructureBlockAction,
				mode: StructureBlockMode,
				name: String,
				offset_x: i8,
				offset_y: i8,
				offset_z: i8,
				size_x: i8,
				size_y: i8,
				size_z: i8,
				mirror: StructureBlockMirror,
				rotation: SpecialBlockRotation,
				metadata: String,
				#[doc = "Between 0 and 1"]
				integrity: f32,
				seed: VarLong,
				flags: StructureBlockFlags
			},
			SetTestBlock, 0x3A #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Set_Test_Block>"] => {
				position: Position,
				mode: TestBlockMode,
				message: String
			},
			UpdateSign, 0x3B #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Update_Sign>"] => {
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
			TeleportToEntity, 0x3D #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Teleport_To_Entity>"] => {
				target_player: Uuid
			},
			TestInstanceBlockAction, 0x3E #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Test_Instance_Block_Action>"] => {
				position: Position,
				action: TestInstanceBlockActionAction,
				test: PrefixedOptional<String>,
				size_x: VarInt,
				size_y: VarInt,
				size_z: VarInt,
				rotation: SpecialBlockRotation,
				ignore_entities: bool,
				status: TestInstanceStatus,
				error_message: PrefixedOptional<TextComponent>
			},
			UseItemOn, 0x3F #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Use_Item_On>"] => {
				hand: UseItemHand,
				location: Position,
				face: BlockFace,
				#[doc = "The position of the crosshair on the block, from 0 to 1 increasing from west to east."]
				cursor_position_x: f32,
				#[doc = "The position of the crosshair on the block, from 0 to 1 increasing from bottom to top."]
				cursor_position_y: f32,
				#[doc = "The position of the crosshair on the block, from 0 to 1 increasing from north to south."]
				cursor_position_z: f32,
				inside_block: bool,
				world_border_hit: bool,
				#[doc = "Block change sequence number"]
				sequence: VarInt
			},
			UseItem, 0x40 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Use_Item>"] => {
				hand: UseItemHand,
				sequence: VarInt,
				yaw: f32,
				pitch: f32
			},
			CustomClickAction, 0x41 #[doc = "<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Custom_Click_Action>"] => {
				id: String,
				payload: NbtCompound
			}
		}
	}
});
