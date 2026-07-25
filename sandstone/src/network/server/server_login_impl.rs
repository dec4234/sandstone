use crate::game::player::PlayerGamemode;
use crate::network::CraftConnection;
use crate::network::network_error::NetworkError;
use crate::network::server::server_handler_impl::DefaultServerEncryptionHandler;
use crate::network::server::server_handler_trait::{ServerEncryptionHandler, ServerLoginHandler};
use crate::protocol::game::info::registry::registry_generator;
use crate::protocol::packets::packet_definer::PacketState;
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

/// The default server login handler used to handle the login sequence for a Minecraft server.
///
/// ```ignore
/// DefaultServerLoginHandler::default().handle_login(&mut connection).await.unwrap();
/// ```
#[derive(Debug, Clone, SmartDefault)]
pub struct DefaultServerLoginHandler {
	#[default(true)]
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
				panic!("Expected login start packet, got {login_start:?}");
			}
		};

		DefaultServerEncryptionHandler::handle_encryption(connection, self.should_authenticate_with_mojang, username.clone(), None).await?;

		if let Some(compression_threshold) = self.compression_threshold {
			let set_compression = Packet::SetCompression(SetCompressionPacket::new(VarInt(compression_threshold)));
			connection.send_packet(set_compression).await?;
			connection.enable_compression(Some(compression_threshold as u32));
			debug!("Sent set compression to {connection} and enabled compression");
		}

		let login_success = Packet::LoginSuccess(LoginSuccessPacket::new(Uuid::new_v4(), username, PrefixedArray::new(vec![])));
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

		let packs = Packet::ClientboundKnownPacks(ClientboundKnownPacksPacket::new(PrefixedArray::new(vec![])));
		connection.send_packet(packs).await?;

		debug!("Sent clientbound known packs to {connection}");

		loop {
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
				Packet::ServerboundKnownPacks(..) => {
					debug!("Received serverbound known packs from {connection}: {packs:?}");
					break;
				}
				_ => {
					return Err(NetworkError::ExpectedDifferentPacket(format!("Expected serverbound known packs packet, got {packs:?}")));
				}
			}
		}

		// send all registry packets
		debug!("Sending registry packets to {connection}");
		for p in registry_generator::default() {
			connection.send_packet(p).await?;
		}

		let tags = Packet::UpdateTags(UpdateTagsPacket::new(PrefixedArray::new(vec![Mapping {
			key: "minecraft:timeline".to_string(),
			value: PrefixedArray::new(vec![Tag {
				identifier: "minecraft:in_overworld".to_string(),
				entries: PrefixedArray::new(vec![VarInt(0), VarInt(2)]),
			}]),
		}])));
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
			PlayerGamemode::UNDEFINED,
			false,
			true,
			false,
			None,
			None,
			VarInt(0),
			VarInt(63),
			false,
		));

		debug!("Sending login info packet {login:?}");
		connection.send_packet(login).await?;

		let sync = Packet::SyncPlayerPosition(SyncPlayerPositionPacket::new(VarInt(2), 0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, TeleportFlags::default()));
		connection.send_packet(sync).await?;

		debug!("Sent sync player position to {connection}");

		let telep = connection.receive_packet().await?;
		match telep {
			Packet::ConfirmTeleport(..) => {
				debug!("Received teleport confirm from {connection}");
				debug!("Teleport confirm: {telep:?}");
			}
			_ => {
				debug!("Expected teleport confirm, got {telep:?}");
				return Err(NetworkError::ExpectedDifferentPacket(format!("Expected teleport confirm packet, got {telep:?}")));
			}
		}

		let setpos = connection.receive_packet().await?;
		match setpos {
			Packet::SetPlayerPositionRotation(..) => {
				debug!("Received set player position rotation from {connection}");
				debug!("Set player position rotation: {setpos:?}");
			}
			_ => {
				debug!("Expected set player position rotation, got {setpos:?}");
				return Err(NetworkError::ExpectedDifferentPacket(format!("Expected set player position rotation packet, got {setpos:?}")));
			}
		}

		Ok(())
	}
}