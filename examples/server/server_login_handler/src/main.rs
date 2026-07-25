//! https://minecraft.wiki/w/Java_Edition_protocol/FAQ#What%27s_the_normal_login_sequence_for_a_client%3F

use log::{LevelFilter, debug};
use simple_logger::SimpleLogger;
use std::sync::Arc;
use tokio::net::TcpListener;

use sandstone::network::CraftConnection;
use sandstone::network::network_error::NetworkError;
use sandstone::network::server::server_handler_impl::{DefaultServerHandshakeHandler, DefaultServerPingHandler, DefaultServerStatusHandler};
use sandstone::network::server::server_handler_trait::{ServerHandshakeHandler, ServerLoginHandler, ServerStatusHandler};
use sandstone::network::server::server_login_impl::DefaultServerLoginHandler;
use sandstone::protocol::game::world::generator::superflat;
use sandstone::protocol::packets::packet_definer::{PacketDirection, PacketState};
use sandstone::protocol::packets::packet_parts::GameEventType;
use sandstone::protocol::packets::packet_parts::status::{PlayerSample, StatusResponseSpec};
use sandstone::protocol::packets::{
	ChunkBatchFinishedPacket, ChunkBatchStartPacket, ChunkDataUpdateLightPacket, ClientboundKeepAlivePacket, GameEventPacket, Packet, SetCenterChunkPacket, StatusResponsePacket,
};
use sandstone::protocol_types::datatypes::var_types::VarInt;
use sandstone::protocol_types::protocol_verison::ProtocolVerison;
use std::time::Duration;
use tokio::sync::Mutex;

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

		DefaultServerLoginHandler::default().handle_login(&mut client).await.unwrap();

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
