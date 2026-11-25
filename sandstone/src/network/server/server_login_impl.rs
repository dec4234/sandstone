use crate::game::player::{PlayerGamemode, PreviousGamemode};
use crate::network::CraftConnection;
use crate::network::network_error::NetworkError;
use crate::network::server::server_handler_impl::DefaultServerEncryptionHandler;
use crate::network::server::server_handler_trait::{ServerEncryptionHandler, ServerLoginHandler};
use crate::protocol::game::info::registry::registry_generator;
use crate::protocol::game::info::registry::vanilla_registries::{VANILLA_TAGS, VANILLA_VERSION};
use crate::protocol::packets::packet_definer::PacketState;
use crate::protocol::packets::packet_parts::ResourcePackEntry;
use crate::protocol::packets::packet_parts::Tag;
use crate::protocol::packets::packet_parts::player::TeleportFlags;
use crate::protocol::packets::{ClientboundKnownPacksPacket, ClientboundPluginMessagePacket, FinishConfigurationPacket, LoginInfoPacket, LoginSuccessPacket, Packet, SetCompressionPacket, SyncPlayerPositionPacket, UpdateTagsPacket};
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::serialization::{McSerialize, McSerializer};
use crate::protocol_types::datatypes::internal_types::Mapping;
use crate::protocol_types::datatypes::var_types::VarInt;
use log::debug;
use rand::Rng;
use smart_default::SmartDefault;
use uuid::Uuid;

/// Every damage type tag in the vanilla 26.3 data pack.
const VANILLA_DAMAGE_TYPE_TAGS: [&str; 36] = [
	"always_hurts_ender_dragons",
	"always_kills_armor_stands",
	"always_most_significant_fall",
	"always_triggers_silverfish",
	"avoids_guardian_thorns",
	"burn_from_stepping",
	"burns_armor_stands",
	"bypasses_armor",
	"bypasses_cooldown",
	"bypasses_effects",
	"bypasses_enchantments",
	"bypasses_invulnerability",
	"bypasses_resistance",
	"bypasses_shield",
	"bypasses_wolf_armor",
	"can_break_armor_stand",
	"damages_helmet",
	"ignites_armor_stands",
	"is_drowning",
	"is_explosion",
	"is_fall",
	"is_fire",
	"is_freezing",
	"is_lightning",
	"is_player_attack",
	"is_projectile",
	"mace_smash",
	"no_anger",
	"no_impact",
	"no_knockback",
	"no_wolf_retaliation",
	"panic_causes",
	"panic_environmental_causes",
	"sulfur_cube_with_block_immune_to",
	"witch_resistant_to",
	"wither_immune_to",
];

/// The default server login handler used to handle the login sequence for a Minecraft server.
///
/// ```ignore
/// DefaultServerLoginHandler::default().handle_login(&mut connection).await.unwrap();
/// ```
#[derive(Debug, Clone, SmartDefault)]
pub struct DefaultServerLoginHandler {
	/// Defaults to true with the `auth` feature and false without it.
	#[default(cfg!(feature = "auth"))]
	pub should_authenticate_with_mojang: bool,
	#[default(Some(400))]
	pub compression_threshold: Option<i32>,
	#[default("sandstone".to_string())]
	pub server_brand: String,
	#[default(20)]
	pub max_players: i32,
	#[default(10)]
	pub view_distance: i32,
	#[default(10)]
	pub simulation_distance: i32
}

impl ServerLoginHandler for DefaultServerLoginHandler {
	async fn handle_login(&self, connection: &mut CraftConnection) -> Result<(), NetworkError> {
		debug!("Beginning login sequence for {connection}");

		let login_start = connection.receive_packet().await?;
		let username = match login_start {
			Packet::LoginStart(packet) => {
				debug!("Received login start packet from {connection}");
				debug!("Login start packet: {packet:?}");

				packet.username
			}
			_ => {
				return Err(NetworkError::ExpectedDifferentPacket(format!("Expected login start packet, got {login_start:?}")));
			}
		};

		DefaultServerEncryptionHandler::handle_encryption(connection, self.should_authenticate_with_mojang, username.clone(), None).await?;

		if let Some(compression_threshold) = self.compression_threshold {
			let set_compression = Packet::SetCompression(SetCompressionPacket::new(VarInt(compression_threshold)));
			connection.send_packet(set_compression).await?;
			connection.enable_compression(Some(compression_threshold as u32));
			debug!("Sent set compression to {connection} and enabled compression");
		}

		let login_success = Packet::LoginSuccess(LoginSuccessPacket::new(Uuid::new_v4(), username, PrefixedArray::new(vec![]), Uuid::new_v4()));
		connection.send_packet(login_success).await?;

		let login_ack = connection.receive_packet().await?;
		match login_ack {
			Packet::LoginAcknowledged(..) => {
				debug!("Received login acknowledged packet from {connection}");
				debug!("Login acknowledged packet: {login_ack:?}");
				connection.change_state(PacketState::CONFIGURATION);
			}
			_ => {
				return Err(NetworkError::ExpectedDifferentPacket(format!("Expected login acknowledged packet, got {login_ack:?}")));
			}
		}

		let mut serializer = McSerializer::new();
		self.server_brand.mc_serialize(&mut serializer)?;

		let server_brand = Packet::ClientboundPluginMessage(ClientboundPluginMessagePacket {
			channel: "minecraft:brand".to_string(),
			data: serializer.output,
		});
		connection.send_packet(server_brand).await?;
		debug!("Sent server brand to {connection}");

		let core_pack = ResourcePackEntry {
			namespace: "minecraft".to_string(),
			id: "core".to_string(),
			version: VANILLA_VERSION.to_string(),
		};
		let packs = Packet::ClientboundKnownPacks(ClientboundKnownPacksPacket::new(PrefixedArray::new(vec![core_pack.clone()])));
		connection.send_packet(packs).await?;

		debug!("Sent clientbound known packs to {connection}");

		let client_has_core_pack = loop {
			let packs = connection.receive_packet().await?;
			match packs {
				Packet::ServerboundPluginMessage(..) => {
					// optional: before known packs
					debug!("Received plugin message from {connection}: {packs:?}");
				}
				Packet::ClientInformation(..) => {
					// optional: before known packs
					debug!("Received client information from {connection}: {packs:?}");
				}
				Packet::ServerboundKnownPacks(ref packet) => {
					debug!("Received serverbound known packs from {connection}: {packs:?}");
					break packet.entries.slice().contains(&core_pack);
				}
				_ => {
					return Err(NetworkError::ExpectedDifferentPacket(format!("Expected serverbound known packs packet, got {packs:?}")));
				}
			}
		};

		// With the core pack, send only the vanilla entry IDs and the client uses its own data.
		// Without it, send the minimal registries with full data.
		let registries = if client_has_core_pack {
			registry_generator::vanilla_known_pack()
		} else {
			registry_generator::default()
		};
		debug!("Sending registry packets to {connection} (client has core pack: {client_has_core_pack})");
		for p in registries {
			connection.send_packet(p).await?;
		}

		// Tags are never taken from a known pack, so the client always needs them in full.
		let tags = if client_has_core_pack {
			Packet::UpdateTags(UpdateTagsPacket::new(PrefixedArray::new(
				VANILLA_TAGS
					.iter()
					.map(|(registry, tags)| Mapping {
						key: registry.to_string(),
						value: PrefixedArray::new(
							tags.iter()
								.map(|(tag, ids)| Tag {
									identifier: tag.to_string(),
									entries: PrefixedArray::new(ids.iter().map(|id| VarInt(*id)).collect()),
								})
								.collect(),
						),
					})
					.collect(),
			)))
		} else {
			// The minimum tags that default() needs.
			Packet::UpdateTags(UpdateTagsPacket::new(PrefixedArray::new(vec![
				Mapping {
					key: "minecraft:timeline".to_string(),
					value: PrefixedArray::new(vec![Tag {
						identifier: "minecraft:in_overworld".to_string(),
						entries: PrefixedArray::new(vec![VarInt(0), VarInt(2)]),
					}]),
				},
				// The default overworld dimension type refers to this tag in its "infiniburn" field.
				// The client fails registry loading if the tag is missing. Empty is valid.
				Mapping {
					key: "minecraft:block".to_string(),
					value: PrefixedArray::new(vec![Tag {
						identifier: "minecraft:infiniburn_overworld".to_string(),
						entries: PrefixedArray::new(vec![]),
					}]),
				},
				// The client fails Finish Configuration if any vanilla damage type tag is missing.
				// Empty is valid. The list comes from data/minecraft/tags/damage_type in the 26.3 client jar.
				Mapping {
					key: "minecraft:damage_type".to_string(),
					value: PrefixedArray::new(
						VANILLA_DAMAGE_TYPE_TAGS
							.iter()
							.map(|name| Tag {
								identifier: format!("minecraft:{name}"),
								entries: PrefixedArray::new(vec![]),
							})
							.collect(),
					),
				},
			])))
		};
		connection.send_packet(tags).await?;

		debug!("Sent update tags to {connection}");

		let packet = Packet::FinishConfiguration(FinishConfigurationPacket::new());
		connection.send_packet(packet).await?;

		debug!("Sent finish configuration to {connection}");

		let ack = connection.receive_packet().await?;
		match ack {
			Packet::AcknowledgeFinishConfiguration(..) => {
				debug!("Received acknowledge finish configuration from {connection}");
				debug!("Acknowledge finish configuration: {ack:?}");
			}
			_ => {
				debug!("Expected acknowledge finish configuration, got {ack:?}");
				return Err(NetworkError::ExpectedDifferentPacket(format!("Expected acknowledge finish configuration packet, got {ack:?}")));
			}
		}

		connection.change_state(PacketState::PLAY);

		let login = Packet::LoginInfo(LoginInfoPacket::new(
			rand::rng().next_u32() as i32,
			false,
			PrefixedArray::new(vec!["minecraft:overworld".to_string(), "minecraft:the_nether".to_string(), "minecraft:the_end".to_string()]),
			self.max_players.into(),
			self.view_distance.into(),
			self.simulation_distance.into(),
			false,
			true,
			false,
			VarInt(0),
			"minecraft:overworld".to_string(),
			rand::rng().next_u64() as i64,
			PlayerGamemode::SURVIVAL,
			PreviousGamemode(PlayerGamemode::UNDEFINED),
			false,
			true,
			false,
			None,
			None,
			VarInt(0),
			VarInt(63),
			self.should_authenticate_with_mojang,
			false,
		));

		debug!("Sending login info packet {login:?}");
		connection.send_packet(login).await?;

		let sync = Packet::SyncPlayerPosition(SyncPlayerPositionPacket::new(VarInt(2), 0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, TeleportFlags::default()));
		connection.send_packet(sync).await?;

		debug!("Sent sync player position to {connection}");

		// Do not wait for Confirm Teleportation here. The client can send other PLAY packets before
		// it (a Fabric client sends a "minecraft:register" plugin message first). The caller reads
		// all PLAY packets, so it gets the Confirm Teleportation with them.
		//
		// Do not wait for Set Player Position and Rotation here. The client stays on the loading
		// screen and does not send it until it has chunks, and the caller sends chunks after this
		// function returns.

		Ok(())
	}
}