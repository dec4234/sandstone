use std::cmp::PartialEq;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Error, Result};
use log::{debug, trace};

use crate::network::connection::CraftClient;
use crate::network::network_error;
use crate::packets::packet_definer::PacketState;
use crate::packets::packets::packet::{Packet, PingResponseBody, StatusRequestBody, StatusResponseBody};
use crate::packets::serialization::serializer_error::SerializingErr::InvalidPacketState;
use crate::protocol_details::datatypes::var_types::VarInt;

/// Lists the methods required to handle a status request. Check [DefaultStatusHandler] for a default implementation.
///
/// The status procedure can be found [here](https://wiki.vg/Server_List_Ping)
pub trait StatusHandler {
	async fn handle_status<P: PingHandler>(connection: &mut CraftClient, status_response: StatusResponseBody, ping_handler: P) -> Result<()>;
}

/// Lists the methods required to handle a ping request. Check [DefaultPingHandler] for a default implementation.
///
/// The ping procedure can be found [here](https://wiki.vg/Server_List_Ping)
pub trait PingHandler {
	async fn handle_ping(connection: &mut CraftClient) -> Result<()>;
}

/// The default server-list status handler. Not sure why you wouldn't want to use it, but it's here.
pub struct DefaultStatusHandler;

impl StatusHandler for DefaultStatusHandler {
	async fn handle_status<P: PingHandler>(connection: &mut CraftClient, status_response: StatusResponseBody, _ping_handler: P) -> Result<()> {
		if connection.packet_state != PacketState::STATUS {
			return Err(Error::from(InvalidPacketState));
		}

		debug!("Handling status for {}", connection);

		let packet = connection.receive_packet().await?;

		match packet {
			Packet::StatusRequest(s) => {
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
				return Err(anyhow::anyhow!("Invalid packet received, expected status request"));
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
	async fn handle_ping(connection: &mut CraftClient) -> Result<()> {
		if connection.packet_state != PacketState::STATUS {
			return Err(Error::from(InvalidPacketState));
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

/// The procedure required to handle a handshake. Check [DefaultHandshakeHandler] for a default implementation.
///
/// If you would like to implement it yourself then check [here](https://wiki.vg/Protocol#Handshake)
pub trait HandshakeHandler {
	async fn handle_handshake(client: &mut CraftClient) -> Result<()>;
}


/// The default handshake handler. Not sure why you wouldn't want to use it, but it's here.
pub struct DefaultHandshakeHandler;

impl HandshakeHandler for DefaultHandshakeHandler {
	async fn handle_handshake(client: &mut CraftClient) -> Result<()> {
		if client.packet_state != PacketState::HANDSHAKING {
			return Err(Error::from(network_error::InvalidPacketState));
		}

		let packet = client.receive_packet().await?;

		match packet {
			Packet::Handshaking(handshake) => {
				if handshake.next_state == VarInt(1) {
					client.change_state(PacketState::STATUS);
				} else if handshake.next_state == VarInt(2) {
					client.change_state(PacketState::LOGIN);
				} else {
					return Err(anyhow::anyhow!("Invalid next state detected, got \"{}\"", handshake.next_state.0));
				}
			}
			_ => {
				return Err(anyhow::anyhow!("Invalid packet received, expected handshake"));
			}
		}

		debug!("Handshake complete for {}", client);

		Ok(())
	}
}
