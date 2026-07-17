use crate::network::network_error::NetworkError;
use crate::network::server::server_handler_trait::{ServerEncryptionHandler, ServerHandshakeHandler, ServerPingHandler, ServerStatusHandler};
use crate::network::CraftConnection;
use crate::protocol::packets::packet_definer::PacketState;
use crate::protocol::packets::{EncryptionRequestPacket, Packet, PingResponsePacket, StatusResponsePacket};
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::util::encryption::rsa::{decrypt_with_private_key, encode_public_key, generate_key_pair, generate_verify_token};
use crate::util::encryption::EncryptionError;
use log::{debug, error, info, trace, warn};
use mojang_api::{generate_server_id, has_joined, HasJoinedResponse};
use std::time::{SystemTime, UNIX_EPOCH};

/// The default server-list status handler. No reason to really override this but you could if you wanted to.
/// 
/// ```ignore
/// DefaultServerStatusHandler::handle_status(connection, status_response, DefaultServerPingHandler).await.unwrap();
/// ```
pub struct DefaultServerStatusHandler;

impl ServerStatusHandler for DefaultServerStatusHandler {
	async fn handle_status<P: ServerPingHandler>(connection: &mut CraftConnection, status_response: StatusResponsePacket, _ping_handler: P) -> Result<(), NetworkError> {
		if connection.packet_state != PacketState::STATUS {
			error!("Invalid packet state for status request, got {:?}", connection.packet_state);
			return Err(NetworkError::InvalidPacketState);
		}

		debug!("Handling status for {}", connection);

		let packet = connection.receive_packet().await?;

		match packet {
			Packet::StatusRequest(_) => {
				trace!("Received status request from {}", connection);

				let packed = Packet::StatusResponse(status_response);

				connection.send_packet(packed).await?;
			}
			Packet::PingRequest(b) => {
				let packed = Packet::PingResponse(PingResponsePacket {
					payload: b.payload as u64,
				});

				connection.send_packet(packed).await?;
				connection.close().await;
				return Ok(());
			}
			_ => {
				return Err(NetworkError::ExpectedDifferentPacket("Invalid packet received, expected status request or ping request".to_string()));
			}
		}

		trace!("Sent response to {}", connection);

		P::handle_ping(connection).await?;

		Ok(())
	}
}

/// The default server ping handler. Usually attached to [DefaultServerStatusHandler] to handle ping requests after a status request.
pub struct DefaultServerPingHandler;

impl ServerPingHandler for DefaultServerPingHandler {
	async fn handle_ping(connection: &mut CraftConnection) -> Result<(), NetworkError> {
		if connection.packet_state != PacketState::STATUS {
			error!("Invalid packet state for ping request, got {:?}", connection.packet_state);
			return Err(NetworkError::InvalidPacketState);
		}

		debug!("Handling ping for {}", connection);

		let ping_request = connection.receive_packet().await?;

		match ping_request {
			Packet::PingRequest(_) => {}
			_ => return Err(NetworkError::ExpectedDifferentPacket("Expected ping request packet".to_string())),
		}

		trace!("Received ping request from {}", connection);

		let packed = Packet::PingResponse(PingResponsePacket {
			payload: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
		});

		connection.send_packet(packed).await?;

		trace!("Sent ping to {}", connection);

		connection.close().await;

		Ok(())
	}
}

/// The default handler for the handshake packet. Receives handshake, changes connection state appropriately.
///
/// Example usage:
/// ```ignore
/// DefaultServerHandshakeHandler::handle_handshake(connection).await?;
/// ```
pub struct DefaultServerHandshakeHandler;

impl ServerHandshakeHandler for DefaultServerHandshakeHandler {
	async fn handle_handshake(connection: &mut CraftConnection) -> Result<(), NetworkError> {
		if connection.packet_state != PacketState::HANDSHAKING {
			error!("Invalid packet state for handshake, got {:?}", connection.packet_state);
			return Err(NetworkError::InvalidPacketState);
		}

		let packet = connection.receive_packet().await?;

		match packet {
			Packet::Handshaking(handshake) => {
				if handshake.next_state == VarInt(1) {
					connection.change_state(PacketState::STATUS);
				} else if handshake.next_state == VarInt(2) {
					connection.change_state(PacketState::LOGIN);
				} else if handshake.next_state == VarInt(3) {
					connection.change_state(PacketState::TRANSFER);
				} else {
					return Err(NetworkError::InvalidNextState(format!("Invalid next state detected, got \"{}\"", handshake.next_state.0)));
				}
			}
			_ => {
				return Err(NetworkError::ExpectedDifferentPacket("Invalid packet received, expected handshake".to_string()));
			}
		}

		debug!("Handshake complete for {}", connection);

		Ok(())
	}
}

/// Handle the encryption handshake and Mojang authentication for an incoming client connection.
///
/// Example usage:
/// ```ignore
/// DefaultServerEncryptionHandler::handle_encryption(connection, true, "dec4234".to_string(), None).await?;
/// ```
pub struct DefaultServerEncryptionHandler;

impl ServerEncryptionHandler for DefaultServerEncryptionHandler {
	async fn handle_encryption(
		connection: &mut CraftConnection,
		should_authenticate_with_mojang: bool,
		username: String,
		server_ip: Option<String>,
	) -> Result<Option<HasJoinedResponse>, NetworkError> {
		if connection.packet_state != PacketState::LOGIN {
			error!("Invalid packet state for encryption request, got {:?}", connection.packet_state);
			return Err(NetworkError::InvalidPacketState);
		}

		let pair = generate_key_pair()?;
		let verify_token = generate_verify_token();

		let enc_request = Packet::EncryptionRequest(EncryptionRequestPacket {
			server_id: "".into(),
			public_key: encode_public_key(pair.as_public_key())?.to_vec().into(),
			verify_token: verify_token.to_vec().into(),
			should_authenticate_with_mojang,
		});

		connection.send_packet(enc_request).await?;

		let enc_response = match connection.receive_packet().await? {
			Packet::EncryptionResponse(packet) => packet,
			_ => return Err(NetworkError::ExpectedDifferentPacket("Invalid packet received, expected encryption response".to_string())),
		};

		let shared_secret = decrypt_with_private_key(&pair, enc_response.shared_secret.slice())?;
		let decrypted_verify_token = decrypt_with_private_key(&pair, enc_response.verify_token.slice())?;

		if decrypted_verify_token != verify_token {
			return Err(NetworkError::EncryptionError(EncryptionError::MismatchedVerifyToken {
				expected: verify_token.to_vec(),
				actual: decrypted_verify_token.to_vec(),
			}));
		}

		connection.set_shared_secret(&shared_secret)?;

		if should_authenticate_with_mojang {
			let server_id = generate_server_id("", &encode_public_key(pair.as_public_key())?, &shared_secret);

			debug!("Verifying join with Mojang: username={:?}, server_id={:?}", username, server_id);

			let join_response = match has_joined(username, server_id, server_ip).await {
				Ok(response) => response,
				Err(error) => {
					warn!("Failed to authenticate with Mojang: {:?}", error);
					return Err(NetworkError::HttpError(error));
				}
			};

			info!("Encryption and Mojang authentication complete for {}", connection);
			return Ok(Some(join_response));
		}

		info!("Encryption complete for {} (Mojang authentication skipped)", connection);
		Ok(None)
	}
}
