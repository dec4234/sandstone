//! Tests for the default client and server handlers over a real loopback TCP socket.
//!
//! Each test plays the other side of the connection by hand, in the packet order from the
//! protocol wiki (<https://minecraft.wiki/w/Java_Edition_protocol/FAQ>). A vanilla client
//! disconnects if a packet is out of order, so the order is what we test.

#[cfg(test)]
mod test {
	use sandstone::network::client::client_handlers::ClientStatusHandler;
	use sandstone::network::client::client_status_handler::DefaultClientStatusHandler;
	use sandstone::network::network_error::NetworkError;
	use sandstone::network::server::server_handler_impl::{DefaultServerEncryptionHandler, DefaultServerHandshakeHandler, DefaultServerPingHandler, DefaultServerStatusHandler};
	use sandstone::network::server::server_handler_trait::{ServerEncryptionHandler, ServerHandshakeHandler, ServerLoginHandler, ServerStatusHandler};
	use sandstone::network::server::server_login_impl::DefaultServerLoginHandler;
	use sandstone::network::{ConnectionRole, CraftConnection};
	use sandstone::protocol::packets::packet_definer::PacketState;
	use sandstone::protocol::packets::packet_parts::PlayerPositionFlags;
	use sandstone::protocol::packets::packet_parts::status::StatusResponseSpec;
	use sandstone::protocol::packets::*;
	use sandstone::protocol::serialization::{McDeserialize, McDeserializer};
	use sandstone::protocol_types::datatypes::var_types::VarInt;
	use sandstone::protocol_types::protocol_verison::ProtocolVerison;
	use sandstone::util::encryption::rsa::{decode_public_key, encrypt_with_public_key};
	use std::time::Duration;
	use tokio::net::TcpListener;
	use uuid::Uuid;

	async fn pair() -> (CraftConnection, CraftConnection) {
		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		let addr = listener.local_addr().unwrap();
		let (client, accepted) = tokio::join!(CraftConnection::connect(format!("127.0.0.1:{}", addr.port())), listener.accept());
		let server = CraftConnection::from_connection(accepted.unwrap().0, ConnectionRole::SERVER).unwrap();
		(client.unwrap(), server)
	}

	/// Receive one packet with a timeout, so an order bug fails the test and does not hang it.
	async fn recv(c: &mut CraftConnection) -> Packet {
		tokio::time::timeout(Duration::from_secs(10), c.receive_packet()).await.expect("timed out").unwrap()
	}

	fn handshake(next_state: i32) -> Packet {
		Packet::Handshaking(HandshakingPacket {
			protocol_version: VarInt(ProtocolVerison::latest().get_version_number() as i32),
			server_address: "localhost".into(),
			port: 25565,
			next_state: VarInt(next_state),
		})
	}

	fn status() -> StatusResponseSpec {
		StatusResponseSpec::new(ProtocolVerison::latest(), "&aHello")
	}

	// ---- Handshake ----

	#[tokio::test]
	async fn handshake_moves_to_state_from_next_state() {
		// next_state 1, 2 and 3 select STATUS, LOGIN and TRANSFER. A wrong state makes the server read
		// the next packet with the wrong packet table.
		for (next, expected) in [(1, PacketState::STATUS), (2, PacketState::LOGIN), (3, PacketState::TRANSFER)] {
			let (mut client, mut server) = pair().await;
			client.send_packet(handshake(next)).await.unwrap();
			DefaultServerHandshakeHandler::handle_handshake(&mut server).await.unwrap();
			assert_eq!(server.packet_state, expected);
		}
	}

	#[tokio::test]
	async fn handshake_rejects_unknown_next_state() {
		let (mut client, mut server) = pair().await;
		client.send_packet(handshake(7)).await.unwrap();
		let err = DefaultServerHandshakeHandler::handle_handshake(&mut server).await.unwrap_err();
		assert!(matches!(err, NetworkError::InvalidNextState(_)), "got {err:?}");
	}

	#[tokio::test]
	async fn handshake_requires_handshaking_state() {
		let (_client, mut server) = pair().await;
		server.change_state(PacketState::STATUS);
		let err = DefaultServerHandshakeHandler::handle_handshake(&mut server).await.unwrap_err();
		assert!(matches!(err, NetworkError::InvalidPacketState), "got {err:?}");
	}

	// ---- Status and ping ----

	#[tokio::test]
	async fn client_status_handler_gets_server_status() {
		// End to end: the client handler and the server handlers must agree on the status flow.
		let (mut client, mut server) = pair().await;
		let server_task = tokio::spawn(async move {
			DefaultServerHandshakeHandler::handle_handshake(&mut server).await.unwrap();
			DefaultServerStatusHandler::handle_status(&mut server, StatusResponsePacket::new(status()), DefaultServerPingHandler).await
		});

		let spec = DefaultClientStatusHandler::handle_status(&mut client).await.unwrap();
		assert_eq!(spec, status());
		assert_eq!(client.packet_state, PacketState::STATUS);

		// The server then waits for a ping. Send one so the server task can finish.
		client.send_packet(Packet::PingRequest(PingRequestPacket { payload: 1 })).await.unwrap();
		assert!(matches!(recv(&mut client).await, Packet::PingResponse(_)));
		server_task.await.unwrap().unwrap();
	}

	#[tokio::test]
	async fn client_status_handler_requires_handshaking_state() {
		// A status request after login would put the connection in a broken state.
		let (mut client, _server) = pair().await;
		client.change_state(PacketState::LOGIN);
		let err = DefaultClientStatusHandler::handle_status(&mut client).await.unwrap_err();
		assert!(matches!(err, NetworkError::InvalidPacketState), "got {err:?}");
	}

	#[tokio::test]
	async fn client_status_handler_sends_status_handshake_and_rejects_wrong_reply() {
		// The server must see next_state = 1 and the dialed host, or proxies route the ping wrong.
		let (mut client, mut server) = pair().await;
		let client_task = tokio::spawn(async move { DefaultClientStatusHandler::handle_status(&mut client).await });

		match recv(&mut server).await {
			Packet::Handshaking(h) => {
				assert_eq!(h.next_state, VarInt(1));
				assert_eq!(h.server_address, "127.0.0.1");
			}
			other => panic!("expected handshake, got {other:?}"),
		}
		server.change_state(PacketState::STATUS);
		assert!(matches!(recv(&mut server).await, Packet::StatusRequest(_)));
		// Send the wrong packet back: the client must report an error, not accept it.
		server.send_packet(Packet::PingResponse(PingResponsePacket { payload: 0 })).await.unwrap();
		let err = client_task.await.unwrap().unwrap_err();
		assert!(matches!(err, NetworkError::ExpectedDifferentPacket(_)), "got {err:?}");
	}

	#[tokio::test]
	async fn ping_before_status_request_is_echoed() {
		// Wiki (Ping Request): the server must answer with the same payload. Clients use it to
		// measure latency, so a different value gives a wrong ping.
		let (mut client, mut server) = pair().await;
		client.change_state(PacketState::STATUS);
		server.change_state(PacketState::STATUS);
		client.send_packet(Packet::PingRequest(PingRequestPacket { payload: 123456 })).await.unwrap();
		DefaultServerStatusHandler::handle_status(&mut server, StatusResponsePacket::new(status()), DefaultServerPingHandler).await.unwrap();
		match recv(&mut client).await {
			Packet::PingResponse(p) => assert_eq!(p.payload, 123456),
			other => panic!("expected ping response, got {other:?}"),
		}
	}

	#[tokio::test]
	async fn status_handler_requires_status_state() {
		let (_client, mut server) = pair().await;
		let err = DefaultServerStatusHandler::handle_status(&mut server, StatusResponsePacket::new(status()), DefaultServerPingHandler).await.unwrap_err();
		assert!(matches!(err, NetworkError::InvalidPacketState), "got {err:?}");
	}

	// ---- Encryption ----

	/// Act as the client side of the encryption exchange. Returns the shared secret it sent.
	async fn client_encryption(client: &mut CraftConnection, corrupt_token: bool) -> Vec<u8> {
		let request = match recv(client).await {
			Packet::EncryptionRequest(r) => r,
			other => panic!("expected encryption request, got {other:?}"),
		};
		let key = decode_public_key(request.public_key.slice()).unwrap();
		let secret = vec![7u8; 16];
		let mut token = request.verify_token.slice().to_vec();
		if corrupt_token {
			token[0] ^= 0xFF;
		}
		client
			.send_packet(Packet::EncryptionResponse(EncryptionResponsePacket {
				shared_secret: encrypt_with_public_key(&key, &secret).unwrap().into(),
				verify_token: encrypt_with_public_key(&key, &token).unwrap().into(),
			}))
			.await
			.unwrap();
		secret
	}

	#[tokio::test]
	async fn encryption_with_good_token_enables_encryption() {
		let (mut client, mut server) = pair().await;
		client.change_state(PacketState::LOGIN);
		server.change_state(PacketState::LOGIN);
		let server_task = tokio::spawn(async move {
			let r = DefaultServerEncryptionHandler::handle_encryption(&mut server, false, "Steve".into(), None).await;
			(r, server)
		});
		let secret = client_encryption(&mut client, false).await;
		client.set_shared_secret(&secret).unwrap();
		let (result, mut server) = server_task.await.unwrap();
		// No Mojang check was asked for, so there is no profile.
		assert!(result.unwrap().is_none());

		// Both sides now use AES. A packet must go through unchanged.
		server.send_packet(Packet::SetCompression(SetCompressionPacket::new(VarInt(5)))).await.unwrap();
		match recv(&mut client).await {
			Packet::SetCompression(p) => assert_eq!(p.threshold, VarInt(5)),
			other => panic!("expected set compression, got {other:?}"),
		}
	}

	#[tokio::test]
	async fn encryption_rejects_wrong_verify_token() {
		// The verify token proves the client has the server's public key. A wrong token must stop the login.
		let (mut client, mut server) = pair().await;
		client.change_state(PacketState::LOGIN);
		server.change_state(PacketState::LOGIN);
		let server_task = tokio::spawn(async move { DefaultServerEncryptionHandler::handle_encryption(&mut server, false, "Steve".into(), None).await });
		client_encryption(&mut client, true).await;
		let err = server_task.await.unwrap().unwrap_err();
		assert!(matches!(err, NetworkError::EncryptionError(_)), "got {err:?}");
	}

	#[tokio::test]
	async fn encryption_requires_login_state() {
		let (_client, mut server) = pair().await;
		let err = DefaultServerEncryptionHandler::handle_encryption(&mut server, false, "Steve".into(), None).await.unwrap_err();
		assert!(matches!(err, NetworkError::InvalidPacketState), "got {err:?}");
	}

	// ---- Full login ----

	#[tokio::test]
	async fn full_login_sequence_reaches_play() {
		// This is the order a vanilla client expects. If a packet is missing or moved, the client
		// stays on "Joining world" or disconnects.
		let (mut client, mut server) = pair().await;
		client.change_state(PacketState::LOGIN);
		server.change_state(PacketState::LOGIN);
		let handler = DefaultServerLoginHandler {
			should_authenticate_with_mojang: false,
			compression_threshold: Some(64),
			server_brand: "test-brand".into(),
			..Default::default()
		};
		let server_task = tokio::spawn(async move {
			let r = handler.handle_login(&mut server).await;
			(r, server)
		});

		client.send_packet(Packet::LoginStart(LoginStartPacket { username: "Steve".into(), uuid: Uuid::nil() })).await.unwrap();
		let secret = client_encryption(&mut client, false).await;
		client.set_shared_secret(&secret).unwrap();

		match recv(&mut client).await {
			Packet::SetCompression(p) => assert_eq!(p.threshold, VarInt(64)),
			other => panic!("expected set compression, got {other:?}"),
		}
		client.enable_compression(Some(64));

		match recv(&mut client).await {
			Packet::LoginSuccess(p) => assert_eq!(p.username, "Steve"),
			other => panic!("expected login success, got {other:?}"),
		}
		client.send_packet(Packet::LoginAcknowledged(LoginAcknowledgedPacket {})).await.unwrap();
		client.change_state(PacketState::CONFIGURATION);

		match recv(&mut client).await {
			Packet::ClientboundPluginMessage(p) => {
				assert_eq!(p.channel, "minecraft:brand");
				// The brand is a String with a VarInt length prefix.
				assert_eq!(String::mc_deserialize(&mut McDeserializer::new(&p.data)).unwrap(), "test-brand");
			}
			other => panic!("expected brand, got {other:?}"),
		}
		assert!(matches!(recv(&mut client).await, Packet::ClientboundKnownPacks(_)));
		client.send_packet(Packet::ServerboundKnownPacks(ServerboundKnownPacksPacket::new(vec![].into()))).await.unwrap();

		// Registry data and tags come next, then Finish Configuration.
		let mut saw_registry = false;
		let mut saw_tags = false;
		loop {
			match recv(&mut client).await {
				Packet::RegistryData(_) => saw_registry = true,
				Packet::UpdateTags(_) => saw_tags = true,
				Packet::FinishConfiguration(_) => break,
				other => panic!("unexpected packet in configuration: {other:?}"),
			}
		}
		assert!(saw_registry, "the client needs registry data before it can join");
		assert!(saw_tags);
		client.send_packet(Packet::AcknowledgeFinishConfiguration(AcknowledgeFinishConfigurationPacket {})).await.unwrap();
		client.change_state(PacketState::PLAY);

		assert!(matches!(recv(&mut client).await, Packet::LoginInfo(_)));
		let teleport_id = match recv(&mut client).await {
			Packet::SyncPlayerPosition(p) => p.teleport_id,
			other => panic!("expected sync player position, got {other:?}"),
		};
		client.send_packet(Packet::ConfirmTeleport(ConfirmTeleportPacket { teleport_id })).await.unwrap();
		client
			.send_packet(Packet::SetPlayerPositionRotation(SetPlayerPositionRotationPacket {
				x: 0.0,
				y: 10.0,
				z: 0.0,
				yaw: 0.0,
				pitch: 0.0,
				flags: PlayerPositionFlags::new(true, false),
			}))
			.await
			.unwrap();

		let (result, server) = server_task.await.unwrap();
		result.unwrap();
		assert_eq!(server.packet_state, PacketState::PLAY);
		assert_eq!(server.compression_threshold, Some(64));
	}

	#[tokio::test]
	async fn login_without_compression_skips_set_compression() {
		// With no threshold, Login Success must be the first packet after encryption.
		let (mut client, mut server) = pair().await;
		client.change_state(PacketState::LOGIN);
		server.change_state(PacketState::LOGIN);
		let handler = DefaultServerLoginHandler {
			should_authenticate_with_mojang: false,
			compression_threshold: None,
			..Default::default()
		};
		let server_task = tokio::spawn(async move { handler.handle_login(&mut server).await });

		client.send_packet(Packet::LoginStart(LoginStartPacket { username: "Alex".into(), uuid: Uuid::nil() })).await.unwrap();
		let secret = client_encryption(&mut client, false).await;
		client.set_shared_secret(&secret).unwrap();
		assert!(matches!(recv(&mut client).await, Packet::LoginSuccess(_)));

		// Send the wrong packet: the server must return an error, not continue.
		client.send_packet(Packet::LoginStart(LoginStartPacket { username: "Alex".into(), uuid: Uuid::nil() })).await.unwrap();
		let err = server_task.await.unwrap().unwrap_err();
		assert!(matches!(err, NetworkError::ExpectedDifferentPacket(_)), "got {err:?}");
	}

	#[tokio::test]
	async fn login_rejects_wrong_first_packet() {
		// A bad first packet must give an error, so the caller can drop this one connection.
		// A panic here would stop the server task.
		let (mut client, mut server) = pair().await;
		client.change_state(PacketState::LOGIN);
		server.change_state(PacketState::LOGIN);
		client.send_packet(Packet::LoginAcknowledged(LoginAcknowledgedPacket {})).await.unwrap();
		let handler = DefaultServerLoginHandler { should_authenticate_with_mojang: false, ..Default::default() };
		let err = handler.handle_login(&mut server).await.unwrap_err();
		assert!(matches!(err, NetworkError::ExpectedDifferentPacket(_)), "got {err:?}");
	}
}
