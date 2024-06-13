use std::time::{SystemTime, UNIX_EPOCH};

use log::{debug, trace};

use crate::network::client::client_handlers::{HandshakeHandler, PingHandler, StatusHandler};
use crate::network::client::CraftClient;
use crate::network::network_error::NetworkError;
use crate::protocol::packet_definer::PacketState;
use crate::protocol::packets::{Packet, PingResponseBody, StatusResponseBody};
use crate::protocol_types::datatypes::var_types::VarInt;

pub mod status_components;

/*
The purpose of this file is to provide default implementations for the status and ping handlers.
There is no conceivable reason why you would want to override these, but if you do, you can implement
them yourself and use them.
 */

/// The default server-list status handler. Not sure why you wouldn't want to use it, but it's here.
pub struct DefaultStatusHandler;

impl StatusHandler for DefaultStatusHandler {
	async fn handle_status<P: PingHandler>(connection: &mut CraftClient, status_response: StatusResponseBody, _ping_handler: P) -> Result<(), NetworkError> {
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
				let packed = Packet::PingResponse(PingResponseBody {
					payload: b.payload as u64
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

/// The default ping handler. Not sure why you wouldn't want to use it, but it's here.
pub struct DefaultPingHandler;

impl PingHandler for DefaultPingHandler {
	async fn handle_ping(connection: &mut CraftClient) -> Result<(), NetworkError> {
		if connection.packet_state != PacketState::STATUS {
			return Err(NetworkError::InvalidPacketState);
		}

		debug!("Handling ping for {}", connection);

		let ping_request = connection.receive_packet().await;

		if let Err(e) = ping_request {
			return Err(e); // pipe all other errors
		}

		trace!("Received ping request from {}", connection);

		let packed = Packet::PingResponse(PingResponseBody {
			payload: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
		});

		connection.send_packet(packed).await?;

		trace!("Sent ping to {}", connection);

		connection.close().await;

		Ok(())
	}
}

/// The default handshake handler. Not sure why you wouldn't want to use it, but it's here.
pub struct DefaultHandshakeHandler;

impl HandshakeHandler for DefaultHandshakeHandler {
	async fn handle_handshake(client: &mut CraftClient) -> Result<(), NetworkError> {
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
