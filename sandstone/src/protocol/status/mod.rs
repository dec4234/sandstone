//! The purpose of this file is to provide default implementations for the status and ping handlers.
//! There is no conceivable reason why you would want to override these, but if you do, you can implement
//! them yourself and use them.

use std::time::{SystemTime, UNIX_EPOCH};

use log::{debug, trace};

use crate::network::client::client_handlers::{ServerHandshakeHandler, ServerPingHandler, ServerStatusHandler};
use crate::network::network_error::NetworkError;
use crate::network::server::server_handler::ClientStatusHandler;
use crate::network::CraftConnection;
use crate::protocol::packets::packet_definer::PacketState;
use crate::protocol::packets::{HandshakingPacket, Packet, PingResponsePacket, StatusRequestPacket, StatusResponsePacket};
use crate::protocol::status::status_components::StatusResponseSpec;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::protocol_types::protocol_verison::ProtocolVerison;

pub mod status_components;

/// The default server-list status handler. Not sure why you wouldn't want to use it, but it's here.
pub struct DefaultServerStatusHandler;

impl ServerStatusHandler for DefaultServerStatusHandler {
	async fn handle_status<P: ServerPingHandler>(connection: &mut CraftConnection, status_response: StatusResponsePacket, _ping_handler: P) -> Result<(), NetworkError> {
		if connection.packet_state != PacketState::STATUS {
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

/// The default server ping handler. Not sure why you wouldn't want to use it, but it's here.
pub struct DefaultServerPingHandler;

impl ServerPingHandler for DefaultServerPingHandler {
	async fn handle_ping(connection: &mut CraftConnection) -> Result<(), NetworkError> {
		if connection.packet_state != PacketState::STATUS {
			return Err(NetworkError::InvalidPacketState);
		}

		debug!("Handling ping for {}", connection);

		let ping_request = connection.receive_packet().await?;

		match ping_request {
			Packet::PingRequest(_) => {

			}
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

/// The default handshake handler. Not sure why you wouldn't want to use it, but it's here.
pub struct DefaultServerHandshakeHandler;

impl ServerHandshakeHandler for DefaultServerHandshakeHandler {
	async fn handle_handshake(client: &mut CraftConnection) -> Result<(), NetworkError> {
		if client.packet_state != PacketState::HANDSHAKING {
			return Err(NetworkError::InvalidPacketState);
		}

		let packet = client.receive_packet().await?;

		match packet {
			Packet::Handshaking(handshake) => {
				if handshake.next_state == VarInt(1) {
					client.change_state(PacketState::STATUS);
				} else if handshake.next_state == VarInt(2) {
					client.change_state(PacketState::LOGIN);
				} else if handshake.next_state == VarInt(3) {
					client.change_state(PacketState::TRANSFER);
				} else {
					return Err(NetworkError::InvalidNextState(format!("Invalid next state detected, got \"{}\"", handshake.next_state.0)));
				}
			}
			_ => {
				return Err(NetworkError::ExpectedDifferentPacket("Invalid packet received, expected handshake".to_string()));
			}
		}

		debug!("Handshake complete for {}", client);

		Ok(())
	}
}

pub struct DefaultClientStatusHandler;

impl ClientStatusHandler for DefaultClientStatusHandler {
	async fn handle_status(connection: &mut CraftConnection) -> Result<StatusResponseSpec, NetworkError> {
		if connection.packet_state != PacketState::HANDSHAKING {
			return Err(NetworkError::InvalidPacketState);
		}

		let handshake = Packet::Handshaking(HandshakingPacket {
			protocol_version: VarInt(ProtocolVerison::latest().get_version_number() as i32),
			server_address: connection.hostname.clone().unwrap_or_else(|| connection.socket_addr.ip().to_string()),
			port: 25565, // unused
			next_state: VarInt(PacketState::STATUS.get_id().unwrap() as i32),
		});

		connection.send_packet(handshake).await?;

		connection.change_state(PacketState::STATUS);

		let status_request = Packet::StatusRequest(StatusRequestPacket {});

		connection.send_packet(status_request).await?;

		let status_response = connection.receive_packet().await?;

		match status_response {
			Packet::StatusResponse(response) => {
				debug!("Received status response from {}", connection);
				Ok(response.response)
			}
			_ => Err(NetworkError::ExpectedDifferentPacket("Invalid packet received, expected status response".to_string())),
		}
	}
}
