//! API tests for [CraftConnection] over a real loopback TCP socket.
//!
//! These tests check the framing rules from the protocol wiki
//! (<https://minecraft.wiki/w/Java_Edition_protocol/Packets#Packet_format>). A vanilla client or
//! server will drop the connection if the length prefix, compression header or encryption is wrong,
//! so each test checks the bytes or behavior that a real peer depends on.

#[cfg(test)]
mod test {
	use sandstone::network::network_error::NetworkError;
	use sandstone::network::{ConnectionRole, CraftConnection};
	use sandstone::protocol::packets::packet_definer::PacketState;
	use sandstone::protocol::packets::{HandshakingPacket, Packet, PingRequestPacket, PingResponsePacket};
	use sandstone::protocol_types::datatypes::var_types::VarInt;
	use std::time::Duration;
	use tokio::io::{AsyncReadExt, AsyncWriteExt};
	use tokio::net::{TcpListener, TcpStream};

	/// Make a connected (client, server) pair on loopback.
	/// The client is dialed through the public `connect` API, as a real user would do.
	async fn pair() -> (CraftConnection, CraftConnection) {
		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		let addr = listener.local_addr().unwrap();

		let (client, accepted) = tokio::join!(CraftConnection::connect(format!("127.0.0.1:{}", addr.port())), listener.accept());
		let server = CraftConnection::from_connection(accepted.unwrap().0, ConnectionRole::SERVER).unwrap();
		(client.unwrap(), server)
	}

	/// Make a server-side connection whose peer is a raw socket, so a test can read and write raw bytes.
	async fn server_with_raw_peer() -> (TcpStream, CraftConnection) {
		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		let addr = listener.local_addr().unwrap();

		let (raw, accepted) = tokio::join!(TcpStream::connect(addr), listener.accept());
		let server = CraftConnection::from_connection(accepted.unwrap().0, ConnectionRole::SERVER).unwrap();
		(raw.unwrap(), server)
	}

	fn handshake(next_state: i32) -> Packet {
		Packet::Handshaking(HandshakingPacket {
			protocol_version: VarInt(774),
			server_address: "localhost".to_string(),
			port: 25565,
			next_state: VarInt(next_state),
		})
	}

	/// A ping with a payload large enough to pass a compression threshold of 0 or more.
	fn ping(payload: i64) -> Packet {
		Packet::PingRequest(PingRequestPacket { payload })
	}

	/// Read until `n` bytes are available, with a timeout so a framing bug does not hang the test suite.
	async fn read_exact_timeout(stream: &mut TcpStream, n: usize) -> Vec<u8> {
		let mut buf = vec![0u8; n];
		tokio::time::timeout(Duration::from_secs(5), stream.read_exact(&mut buf)).await.expect("timed out").unwrap();
		buf
	}

	#[tokio::test]
	async fn connect_keeps_dialed_host_without_port() {
		// The handshake must carry the host the user typed (no port), so proxies like Velocity can route it.
		let (client, server) = pair().await;
		assert_eq!(client.hostname.as_deref(), Some("127.0.0.1"));
		assert_eq!(client.client_type, ConnectionRole::CLIENT);
		// A server connection does not know the dialed host until the handshake arrives.
		assert_eq!(server.hostname, None);
	}

	#[tokio::test]
	async fn new_connection_starts_in_handshaking_without_compression() {
		// Every Minecraft connection must start in HANDSHAKING, plain and uncompressed.
		let (client, server) = pair().await;
		for c in [&client, &server] {
			assert_eq!(c.packet_state, PacketState::HANDSHAKING);
			assert_eq!(c.compression_threshold, None);
			assert_eq!(c.protocol_version, None);
			assert_eq!(c.get_client_version(), None);
		}
	}

	#[tokio::test]
	async fn handshake_round_trip_client_to_server() {
		let (mut client, mut server) = pair().await;
		client.send_packet(handshake(1)).await.unwrap();
		assert_eq!(server.receive_packet().await.unwrap(), handshake(1));
	}

	#[tokio::test]
	async fn uncompressed_frame_matches_wire_format() {
		// Without compression the frame is: VarInt(length of ID + data) + ID + data.
		let (mut raw, mut server_raw) = server_with_raw_peer().await;
		server_raw.change_state(PacketState::STATUS);
		server_raw.send_packet(Packet::PingResponse(PingResponsePacket { payload: 0x0102030405060708 })).await.unwrap();

		let bytes = read_exact_timeout(&mut raw, 10).await;
		assert_eq!(bytes, vec![9, 0x01, 1, 2, 3, 4, 5, 6, 7, 8]);
	}

	#[tokio::test]
	async fn state_change_selects_packet_table() {
		// Packet ID 0x01 means PingRequest only in STATUS. The receiver must decode by its current state.
		let (mut client, mut server) = pair().await;
		client.change_state(PacketState::STATUS);
		server.change_state(PacketState::STATUS);

		client.send_packet(ping(42)).await.unwrap();
		assert_eq!(server.receive_packet().await.unwrap(), ping(42));

		server.send_packet(Packet::PingResponse(PingResponsePacket { payload: 42 })).await.unwrap();
		assert_eq!(client.receive_packet().await.unwrap(), Packet::PingResponse(PingResponsePacket { payload: 42 }));
	}

	#[tokio::test]
	async fn several_packets_in_one_stream_stay_separate() {
		// TCP can join frames together. The length prefix must split them correctly.
		let (mut client, mut server) = pair().await;
		client.change_state(PacketState::STATUS);
		server.change_state(PacketState::STATUS);

		for i in 0..5 {
			client.send_packet(ping(i)).await.unwrap();
		}
		for i in 0..5 {
			assert_eq!(server.receive_packet().await.unwrap(), ping(i));
		}
	}

	#[tokio::test]
	async fn compression_round_trip_above_and_below_threshold() {
		// The handshake has a long address so it is above the threshold. The ping is below it.
		let (mut client, mut server) = pair().await;
		client.enable_compression(Some(16));
		server.enable_compression(Some(16));

		let big = Packet::Handshaking(HandshakingPacket {
			protocol_version: VarInt(774),
			server_address: "a".repeat(200),
			port: 25565,
			next_state: VarInt(1),
		});
		client.send_packet(big.clone()).await.unwrap();
		assert_eq!(server.receive_packet().await.unwrap(), big);

		client.change_state(PacketState::STATUS);
		server.change_state(PacketState::STATUS);
		client.send_packet(ping(7)).await.unwrap();
		assert_eq!(server.receive_packet().await.unwrap(), ping(7));
	}

	#[tokio::test]
	async fn compressed_frame_below_threshold_has_zero_data_length() {
		// Wiki: below the threshold, Data Length must be 0 and the body is sent uncompressed.
		// A vanilla client rejects a small packet that is compressed.
		let (mut raw, mut server) = server_with_raw_peer().await;
		server.change_state(PacketState::STATUS);
		server.enable_compression(Some(256));
		server.send_packet(Packet::PingResponse(PingResponsePacket { payload: 1 })).await.unwrap();

		let bytes = read_exact_timeout(&mut raw, 11).await;
		assert_eq!(bytes[0], 10, "packet length counts the Data Length byte");
		assert_eq!(bytes[1], 0, "Data Length must be 0 when not compressed");
		assert_eq!(bytes[2], 0x01, "packet ID follows unchanged");
		assert_eq!(&bytes[3..], &1u64.to_be_bytes());
	}

	#[tokio::test]
	async fn compressed_frame_above_threshold_has_uncompressed_length() {
		// Wiki: above the threshold, Data Length is the size of ID + data before zlib.
		let (mut raw, mut server) = server_with_raw_peer().await;
		server.change_state(PacketState::STATUS);
		server.enable_compression(Some(0));
		server.send_packet(Packet::PingResponse(PingResponsePacket { payload: 1 })).await.unwrap();

		let len = read_exact_timeout(&mut raw, 1).await[0] as usize;
		let body = read_exact_timeout(&mut raw, len).await;
		assert_eq!(body[0], 9, "Data Length is the uncompressed ID + payload length");
		assert_eq!(body[1], 0x78, "body must start with a zlib header");
	}

	#[tokio::test]
	async fn encryption_round_trip_with_shared_secret() {
		let secret = [7u8; 16];
		let (mut client, mut server) = pair().await;
		client.set_shared_secret(&secret).unwrap();
		server.set_shared_secret(&secret).unwrap();
		client.change_state(PacketState::STATUS);
		server.change_state(PacketState::STATUS);

		// Send more than one packet: AES/CFB8 is a stream cipher, so the state must carry over.
		for i in 0..3 {
			client.send_packet(ping(i)).await.unwrap();
			assert_eq!(server.receive_packet().await.unwrap(), ping(i));
		}
	}

	#[tokio::test]
	async fn encryption_with_compression_round_trip() {
		// Wiki order: compress first, then encrypt the full frame. Both sides must agree.
		let secret = [3u8; 16];
		let (mut client, mut server) = pair().await;
		for c in [&mut client, &mut server] {
			c.set_shared_secret(&secret).unwrap();
			c.enable_compression(Some(0));
			c.change_state(PacketState::STATUS);
		}

		client.send_packet(ping(99)).await.unwrap();
		assert_eq!(server.receive_packet().await.unwrap(), ping(99));
	}

	#[tokio::test]
	async fn encrypted_bytes_differ_from_plaintext() {
		// If encryption did nothing, the session would leak data and vanilla would fail to decrypt.
		let (mut raw, mut server) = server_with_raw_peer().await;
		server.change_state(PacketState::STATUS);
		server.set_shared_secret(&[9u8; 16]).unwrap();
		server.send_packet(Packet::PingResponse(PingResponsePacket { payload: 0x0102030405060708 })).await.unwrap();

		let bytes = read_exact_timeout(&mut raw, 10).await;
		assert_ne!(bytes, vec![9, 0x01, 1, 2, 3, 4, 5, 6, 7, 8]);
	}

	#[tokio::test]
	async fn shared_secret_must_be_16_bytes() {
		// The protocol uses AES-128, so any other key length is a protocol error.
		let (mut client, _server) = pair().await;
		for len in [0usize, 15, 17, 32] {
			match client.set_shared_secret(&vec![0u8; len]) {
				Err(NetworkError::EncryptionError(_)) => {}
				other => panic!("length {len}: expected EncryptionError, got {other:?}"),
			}
		}
	}

	#[tokio::test]
	async fn peek_does_not_consume_packet() {
		// The server uses peek to detect legacy pings. The next receive must still get the packet.
		let (mut client, mut server) = pair().await;
		client.send_packet(handshake(2)).await.unwrap();

		assert_eq!(server.peek_packet().await.unwrap(), handshake(2));
		assert_eq!(server.receive_packet().await.unwrap(), handshake(2));
	}

	#[tokio::test]
	async fn client_peek_decodes_clientbound_packets() {
		// A client receives CLIENT-direction packets. Peek must use the same packet table as receive.
		let (mut client, mut server) = pair().await;
		client.change_state(PacketState::STATUS);
		server.change_state(PacketState::STATUS);
		let pong = Packet::PingResponse(PingResponsePacket { payload: 5 });
		server.send_packet(pong.clone()).await.unwrap();

		assert_eq!(client.peek_packet().await.unwrap(), pong);
		assert_eq!(client.receive_packet().await.unwrap(), pong);
	}

	#[tokio::test]
	async fn peek_n_bytes_returns_raw_frame() {
		let (mut raw, mut server) = server_with_raw_peer().await;
		raw.write_all(&[1, 2, 3, 4]).await.unwrap();
		tokio::time::sleep(Duration::from_millis(50)).await;
		assert_eq!(server.peek_n_bytes(4).await.unwrap(), vec![1, 2, 3, 4]);
	}

	#[tokio::test]
	async fn peek_is_rejected_while_encrypted() {
		// Peeked bytes cannot be decrypted without advancing the cipher, so peek must refuse.
		let (_client, mut server) = pair().await;
		server.set_shared_secret(&[1u8; 16]).unwrap();
		match server.peek_packet().await {
			Err(NetworkError::PeekNotSupportedWhileEncrypted) => {}
			other => panic!("expected PeekNotSupportedWhileEncrypted, got {other:?}"),
		}
	}

	#[tokio::test]
	async fn length_prefix_longer_than_three_bytes_is_rejected() {
		// Wiki: the packet length VarInt is at most 3 bytes. A longer prefix is malicious or corrupt.
		let (mut raw, mut server) = server_with_raw_peer().await;
		raw.write_all(&[0x80, 0x80, 0x80, 0x01]).await.unwrap();
		match server.receive_packet().await {
			Err(NetworkError::SerializingErr(_)) => {}
			other => panic!("expected SerializingErr, got {other:?}"),
		}
	}

	#[tokio::test]
	async fn peer_closing_mid_packet_reports_no_data() {
		// A frame that says 5 bytes but stops early means the peer went away.
		let (mut raw, mut server) = server_with_raw_peer().await;
		raw.write_all(&[5, 0x00]).await.unwrap();
		raw.shutdown().await.unwrap();
		drop(raw);
		match server.receive_packet().await {
			Err(NetworkError::NoDataReceived) => {}
			other => panic!("expected NoDataReceived, got {other:?}"),
		}
	}

	#[tokio::test]
	async fn unknown_packet_id_is_an_error() {
		// ID 0x7F does not exist in HANDSHAKING. It must not be decoded as some other packet.
		let (mut raw, mut server) = server_with_raw_peer().await;
		raw.write_all(&[1, 0x7F]).await.unwrap();
		assert!(server.receive_packet().await.is_err());
	}

	#[tokio::test]
	async fn try_receive_returns_packet_when_data_is_ready() {
		let (mut client, mut server) = pair().await;
		client.send_packet(handshake(1)).await.unwrap();
		server.tcp_stream.readable().await.unwrap();
		tokio::time::sleep(Duration::from_millis(50)).await;
		assert_eq!(server.try_receive_packet().unwrap(), handshake(1));
	}

	#[tokio::test]
	async fn get_client_version_is_none_for_unknown_protocol_number() {
		// After the handshake the server stores the number. Unknown numbers must give None, not a guess.
		let (_client, mut server) = pair().await;
		server.protocol_version = Some(VarInt(-12345));
		assert_eq!(server.get_client_version(), None);
	}

	#[tokio::test]
	async fn display_shows_peer_address() {
		let (client, server) = pair().await;
		assert_eq!(format!("{client}"), format!("CraftConnection: {}", client.socket_addr));
		assert!(format!("{server}").starts_with("CraftConnection: 127.0.0.1:"));
	}

	#[tokio::test]
	async fn close_shuts_down_the_socket() {
		// After close, the peer must see end of stream so it can release its resources.
		let (mut client, mut server) = pair().await;
		assert!(client.close().await);
		let mut buf = [0u8; 1];
		let n = tokio::time::timeout(Duration::from_secs(5), server.tcp_stream.read(&mut buf)).await.unwrap().unwrap();
		assert_eq!(n, 0);
	}
}
