use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use log::{debug, trace};

use crate::network::connection::CraftClient;
use crate::packets::raw_packet::PackagedPacket;
use crate::packets::status::status_packets::{UniversalPingRequest, UniversalPingResponse, UniversalStatusRequest, UniversalStatusResponse};
use crate::protocol_details::datatypes::var_types::VarInt;
use crate::protocol_details::protocol_verison::ProtocolVerison;

pub trait StatusHandler: PingHandler {
	async fn handle_status(&self, connection: &mut CraftClient) -> Result<()>;
}

pub trait PingHandler {
	async fn handle_ping(&self, connection: &mut CraftClient) -> Result<()>;
}

pub struct DefaultStatusHandler;

impl StatusHandler for DefaultStatusHandler {
	async fn handle_status(&self, connection: &mut CraftClient) -> Result<()> {
		debug!("Handling status for {}", connection);

		if connection.peek_next_packet_details().await?.1.0 == 0x01 {
			self.handle_ping(connection).await?;
			return Ok(());
		}

		connection.receive_packet::<UniversalStatusRequest>().await?;

		trace!("Received status request from {}", connection);

		let response = UniversalStatusResponse::new(ProtocolVerison::v1_20, "Hello".to_string());

		let packed = PackagedPacket::new(VarInt(0x00), response);

		connection.send_packet(packed).await?;

		trace!("Sent response to {}", connection);

		self.handle_ping(connection).await?;

		Ok(())
	}
}

impl PingHandler for DefaultStatusHandler {
	async fn handle_ping(&self, connection: &mut CraftClient) -> Result<()> {
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