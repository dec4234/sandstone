use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Error, Result};
use log::{debug, trace};

use crate::network::connection::CraftClient;
use crate::packets::packet_definer::PacketState;
use crate::packets::raw_packet::PackagedPacket;
use crate::packets::serialization::serializer_error::SerializingErr::InvalidPacketState;
use crate::packets::status::status_packets::{UniversalPingRequest, UniversalPingResponse, UniversalStatusRequest, UniversalStatusResponse};
use crate::protocol_details::datatypes::var_types::VarInt;
use crate::protocol_details::protocol_verison::ProtocolVerison;

pub trait StatusHandler {
	async fn handle_status<P: PingHandler>(connection: &mut CraftClient, status_response: UniversalStatusResponse, ping_handler: P) -> Result<()>;
}

pub trait PingHandler {
	async fn handle_ping(connection: &mut CraftClient) -> Result<()>;
}

pub struct DefaultStatusHandler;

impl StatusHandler for DefaultStatusHandler {
	async fn handle_status<P: PingHandler>(connection: &mut CraftClient, status_response: UniversalStatusResponse, ping_handler: P) -> Result<()> {
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