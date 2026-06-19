//! https://minecraft.wiki/w/Java_Edition_protocol/FAQ#What%27s_the_normal_login_sequence_for_a_client%3F

use log::{debug, LevelFilter};
use simple_logger::SimpleLogger;
use std::sync::Arc;
use tokio::net::TcpListener;

use sandstone::game::player::PlayerGamemode;
use sandstone::network::client::client_handlers::{ServerHandshakeHandler, ServerStatusHandler};
use sandstone::network::network_error::NetworkError;
use sandstone::network::CraftConnection;
use sandstone::protocol::game::info::registry::registry_generator;
use sandstone::protocol::game::world::generator::superflat;
use sandstone::protocol::packets::packet_component::{GameEventType, Tag};
use sandstone::protocol::packets::packet_definer::{PacketDirection, PacketState};
use sandstone::protocol::packets::{ChunkBatchFinishedPacket, ChunkBatchStartPacket, ChunkDataUpdateLightPacket, ClientboundKeepAlivePacket, ClientboundKnownPacksPacket, FinishConfigurationPacket, GameEventPacket, LoginInfoPacket, LoginSuccessPacket, Packet, SetCenterChunkPacket, SetCompressionPacket, StatusResponsePacket, SyncPlayerPositionPacket, UpdateTagsPacket};
use sandstone::protocol::serialization::serializer_types::PrefixedArray;
use sandstone::protocol::status::status_components::{PlayerSample, StatusResponseSpec};
use sandstone::protocol::status::{DefaultServerHandshakeHandler, DefaultServerPingHandler, DefaultServerStatusHandler};
use sandstone::protocol_types::datatypes::internal_types::Mapping;
use sandstone::protocol_types::datatypes::var_types::VarInt;
use sandstone::protocol_types::protocol_verison::ProtocolVerison;
use sandstone::util::java::bitfield::BitField;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

#[tokio::main]
async fn main() {
	SimpleLogger::new().with_level(LevelFilter::Debug).init().unwrap();
	debug!("Starting server");

	let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

	let mut response = StatusResponseSpec::new(ProtocolVerison::latest(), "&a&lThis is a test description &b§kttt");
	response.set_player_info(1, 0, vec![PlayerSample::new_random("&6&lTest")]);

	loop {
		let (socket, _) = server.accept().await.unwrap();

		let mut client = CraftConnection::from_connection(socket, PacketDirection::SERVER).unwrap();

		DefaultServerHandshakeHandler::handle_handshake(&mut client).await.unwrap();

		if client.packet_state == PacketState::STATUS {
			DefaultServerStatusHandler::handle_status(&mut client, StatusResponsePacket::new(response.clone()), DefaultServerPingHandler)
				.await
				.unwrap();
			continue;
		}

		debug!("Beginning login sequence for {client}");

		let login_start = client.receive_packet().await.unwrap();
		match login_start {
			Packet::LoginStart(packet) => {
				debug!("Received login start packet from {client}");
				debug!("Login start packet: {packet:?}");

				// todo: extract player details like username
			}
			_ => {
				debug!("Expected login start packet, got {login_start:?}");
				continue;
			}
		}

		let set_compression = Packet::SetCompression(SetCompressionPacket::new(VarInt(400)));
		client.send_packet(set_compression).await.unwrap();
		client.enable_compression(Some(400));
		debug!("Sent set compression to {client} and enabled compression");

		let login_success = Packet::LoginSuccess(LoginSuccessPacket::new(Uuid::new_v4(), "TestUser".to_string(), PrefixedArray::new(vec![])));
		client.send_packet(login_success).await.unwrap();

		let login_ack = client.receive_packet().await.unwrap();
		match login_ack {
			Packet::LoginAcknowledged(..) => {
				debug!("Received login acknowledged packet from {client}");
				debug!("Login acknowledged packet: {login_ack:?}");
				client.change_state(PacketState::CONFIGURATION);
			}
			_ => {
				debug!("Expected login acknowledged, got {login_ack:?}");
				continue;
			}
		}

		let packs = Packet::ClientboundKnownPacks(ClientboundKnownPacksPacket::new(PrefixedArray::new(vec![])));
		client.send_packet(packs).await.unwrap();

		debug!("Sent clientbound known packs to {client}");

		loop {
			let packs = client.receive_packet().await.unwrap();
			match packs {
				Packet::ServerboundPluginMessage(..) => {
					// optional: before known packs
					debug!("Received plugin message from {client}");
					debug!("Plugin message: {packs:?}");
					continue;
				}
				Packet::ClientInformation(..) => {
					// optional: before known packs
					debug!("Received client information from {client}");
					debug!("Client information: {packs:?}");
					continue;
				}
				Packet::ServerboundKnownPacks(..) => {
					debug!("Received serverbound known packs from {client}");
					debug!("Serverbound known packs: {packs:?}");
					break;
				}
				_ => {
					debug!("Expected serverbound known packs, got {packs:?}");
					break;
				}
			}
		}

		// send all registry packets
		debug!("Sending registry packets to {client}");
		for p in registry_generator::default() {
			client.send_packet(p).await.unwrap();
		}

		let tags = Packet::UpdateTags(UpdateTagsPacket::new(PrefixedArray::new(vec![Mapping {
			key: "minecraft:timeline".to_string(),
			value: PrefixedArray::new(vec![Tag {
				identifier: "minecraft:in_overworld".to_string(),
				entries: PrefixedArray::new(vec![VarInt(0), VarInt(2)]),
			}]),
		}])));
		client.send_packet(tags).await.unwrap();

		debug!("Sent update tags to {client}");

		let packet = Packet::FinishConfiguration(FinishConfigurationPacket::new());
		client.send_packet(packet).await.unwrap();

		debug!("Sent finish configuration to {client}");

		let ack = client.receive_packet().await.unwrap();
		match ack {
			Packet::AcknowledgeFinishConfiguration(..) => {
				debug!("Received acknowledge finish configuration from {client}");
				debug!("Acknowledge finish configuration: {ack:?}");
			}
			_ => {
				debug!("Expected acknowledge finish configuration, got {ack:?}");
				continue;
			}
		}

		client.change_state(PacketState::PLAY);

		let login = Packet::LoginInfo(LoginInfoPacket::new(
			9,
			false,
			PrefixedArray::new(vec!["minecraft:overworld".to_string(), "minecraft:the_nether".to_string(), "minecraft:the_end".to_string()]),
			20.into(),
			10.into(),
			10.into(),
			false,
			true,
			false,
			VarInt(0),
			"minecraft:overworld".to_string(),
			-4546743471931916645i64,
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
		client.send_packet(login).await.unwrap();

		let sync = Packet::SyncPlayerPosition(SyncPlayerPositionPacket::new(VarInt(2), 0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, BitField::new(0)));
		client.send_packet(sync).await.unwrap();

		debug!("Sent sync player position to {client}");

		let telep = client.receive_packet().await.unwrap();
		match telep {
			Packet::ConfirmTeleport(..) => {
				debug!("Received teleport confirm from {client}");
				debug!("Teleport confirm: {telep:?}");
			}
			_ => {
				debug!("Expected teleport confirm, got {telep:?}");
				continue;
			}
		}

		let setpos = client.receive_packet().await.unwrap();
		match setpos {
			Packet::SetPlayerPositionRotation(..) => {
				debug!("Received set player position rotation from {client}");
				debug!("Set player position rotation: {setpos:?}");
			}
			_ => {
				debug!("Expected set player position rotation, got {setpos:?}");
				continue;
			}
		}

		// Tell the client to begin loading terrain, then send the chunks around spawn.
		let game_event = Packet::GameEvent(GameEventPacket::new(GameEventType::StartWaitingForLevelChunks, 0.0));
		client.send_packet(game_event).await.unwrap();

		let center = Packet::SetCenterChunk(SetCenterChunkPacket::new(VarInt(0), VarInt(0)));
		client.send_packet(center).await.unwrap();

		// Every column in a superflat world is identical, so generate one chunk and reuse it.
		let (chunk_data, light_data) = superflat::superflat_chunk();

		// Send the full view-distance grid (view_distance = 10 -> 21x21 chunks centered on spawn),
		// wrapped in a chunk batch.
		const VIEW_DISTANCE: i32 = 10;
		let chunk_count = (2 * VIEW_DISTANCE + 1) * (2 * VIEW_DISTANCE + 1);

		client.send_packet(Packet::ChunkBatchStart(ChunkBatchStartPacket::new())).await.unwrap();

		for cx in -VIEW_DISTANCE..=VIEW_DISTANCE {
			for cz in -VIEW_DISTANCE..=VIEW_DISTANCE {
				let chunk = Packet::ChunkDataUpdateLight(ChunkDataUpdateLightPacket::new(cx, cz, chunk_data.clone(), light_data.clone()));
				client.send_packet(chunk).await.unwrap();
			}
		}

		client.send_packet(Packet::ChunkBatchFinished(ChunkBatchFinishedPacket::new(VarInt(chunk_count)))).await.unwrap();

		debug!("Sent {chunk_count} superflat chunks to {client}");

		let arc = Arc::new(Mutex::new(client));

		let clone = arc.clone();
		let mut g1 = tokio::spawn(async move {
			let mut keep_alive_id = 0i64;

			loop {
				tokio::time::sleep(Duration::from_secs(10)).await;

				keep_alive_id += 1;
				let ka = Packet::ClientboundKeepAlive(ClientboundKeepAlivePacket::new(keep_alive_id));
				if let Err(e) = clone.lock().await.send_packet(ka).await {
					debug!("Keep-alive send failed, connection closed: {e:?}");
					break;
				}
			}
		});

		let clone = arc.clone();
		let mut g2 = tokio::spawn(async move {
			loop {
				let result = clone.lock().await.try_receive_packet();

				match result {
					Ok(packet) => match packet {
						Packet::ClientTickEnd(_) => {
							// no console spam
						}
						_ => {
							debug!("Received play packet: {packet:?}");
						}
					},
					Err(NetworkError::IOError(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
						tokio::time::sleep(Duration::from_millis(50)).await;
					}
					// Graceful close by the peer (read returned 0 bytes) or any other error.
					Err(e) => {
						debug!("Connection closed by client: {e:?}");
						break;
					}
				}
			}
		});

		// Shut down the connection when either task finishes
		tokio::select! {
			_ = &mut g1 => { g2.abort(); }
			_ = &mut g2 => { g1.abort(); }
		}

		arc.lock().await.close().await;
	}
}
