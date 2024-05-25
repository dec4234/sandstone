use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Error, Result};
use log::{debug, trace};

use crate::network::connection::CraftClient;
use crate::network::network_error;
use crate::packets::packet_definer::PacketState;
use crate::packets::raw_packet::PackagedPacket;
use crate::packets::serialization::serializer_error::SerializingErr::InvalidPacketState;
use crate::packets::status::status_packets::{UniversalHandshakePacket, UniversalPingRequest, UniversalPingResponse, UniversalStatusRequest, UniversalStatusResponse};
use crate::protocol_details::datatypes::var_types::VarInt;
use crate::protocol_details::protocol_verison::ProtocolVerison;

/// Lists the methods required to handle a status request. Check [DefaultStatusHandler] for a default implementation.
pub trait StatusHandler {
	async fn handle_status<P: PingHandler>(connection: &mut CraftClient, status_response: UniversalStatusResponse, ping_handler: P) -> Result<()>;
}

/// Lists the methods required to handle a ping request. Check [DefaultPingHandler] for a default implementation.
pub trait PingHandler {
	async fn handle_ping(connection: &mut CraftClient) -> Result<()>;
}

/// The default server-list status handler. Not sure why you wouldn't want to use it, but it's here.
pub struct DefaultStatusHandler;

impl StatusHandler for DefaultStatusHandler {
	async fn handle_status<P: PingHandler>(connection: &mut CraftClient, status_response: UniversalStatusResponse, _ping_handler: P) -> Result<()> {
		if connection.packet_state != PacketState::STATUS {
			return Err(Error::from(InvalidPacketState));
		}
		
		debug!("Handling status for {}", connection);

		if connection.peek_next_packet_details().await?.1.0 == 0x01 {
			P::handle_ping(connection).await?;
			return Ok(());
		}

		connection.receive_packet::<UniversalStatusRequest>().await?;

		trace!("Received status request from {}", connection);
		
		let packed = PackagedPacket::new(VarInt(0x00), status_response);

		connection.send_packet(packed).await?;

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

		let ping_request = connection.receive_packet::<UniversalPingRequest>().await;

		if let Err(e) = ping_request {
			return Err(e); // pipe all other errors
		}

		trace!("Received ping request from {}", connection);

		let packed = PackagedPacket::new(VarInt(0x01), UniversalPingResponse {
			payload: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
		});

		connection.send_packet(packed).await?;

		trace!("Sent ping to {}", connection);

		connection.close().await;

		Ok(())
	}
}

pub trait HandshakeHandler {
	async fn handle_handshake(client: &mut CraftClient) -> Result<()>;
}

pub struct DefaultHandshakeHandler;

impl HandshakeHandler for DefaultHandshakeHandler {
	async fn handle_handshake(client: &mut CraftClient) -> Result<()> {
		if client.packet_state != PacketState::HANDSHAKING {
			return Err(Error::from(network_error::InvalidPacketState));
		}
		
		let packet = client.receive_packet::<UniversalHandshakePacket>().await?;
		
		if packet.data.next_state == VarInt(1) {
			client.change_state(PacketState::STATUS);
		} else if packet.data.next_state == VarInt(2) {
			client.change_state(PacketState::LOGIN);
		} else {
			return Err(anyhow::anyhow!("Invalid next state detected, got \"{}\"", packet.data.next_state.0));
		}
		
		Ok(())
	}
}
