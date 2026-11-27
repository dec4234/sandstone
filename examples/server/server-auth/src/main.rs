use log::{LevelFilter, debug, info};
use sandstone::network::CraftConnection;
use sandstone::network::server::server_handler_impl::{DefaultServerEncryptionHandler, DefaultServerHandshakeHandler, DefaultServerPingHandler, DefaultServerStatusHandler};
use sandstone::network::server::server_handler_trait::{ServerEncryptionHandler, ServerHandshakeHandler, ServerStatusHandler};
use sandstone::network::server::server_login_impl::LoginProfile;
use sandstone::protocol::packets::packet_definer::{PacketDirection, PacketState};
use sandstone::protocol::packets::packet_parts::status::{PlayerSample, StatusResponseSpec};
use sandstone::protocol::packets::{LoginSuccessPacket, Packet, SetCompressionPacket, StatusResponsePacket};
use sandstone::protocol::serialization::serializer_types::PrefixedArray;
use sandstone::protocol_types::datatypes::var_types::VarInt;
use sandstone::protocol_types::protocol_verison::ProtocolVerison;
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;

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
		let username = match login_start {
			Packet::LoginStart(packet) => {
				debug!("Received login start packet from {client}");
				debug!("Login start packet: {packet:?}");

				packet.username
			}
			_ => {
				panic!("Expected login start packet, got {login_start:?}");
			}
		};

		// With Mojang authentication on, the handler returns the player's Mojang profile.
		let auth = DefaultServerEncryptionHandler::handle_encryption(&mut client, true, username, None).await.unwrap();
		let profile = LoginProfile::try_from(auth.expect("Mojang authentication was requested")).unwrap();

		let set_compression = Packet::SetCompression(SetCompressionPacket::new(VarInt(400)));
		client.send_packet(set_compression).await.unwrap();
		client.enable_compression(Some(400));
		debug!("Sent set compression to {client} and enabled compression");

		let login_success = Packet::LoginSuccess(LoginSuccessPacket::new(profile.uuid, profile.username, PrefixedArray::new(profile.properties), profile.uuid));
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

		info!("Login sequence complete, encryption enabled.");
		client.close().await;
		break;
	}
}
