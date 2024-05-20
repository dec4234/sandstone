use anyhow::Result;
use log::{debug, trace};

use crate::network::connection::CraftClient;
use crate::packets::raw_packet::PackagedPacket;
use crate::packets::status::status_packets::{UniversalPingRequest, UniversalPingResponse, UniversalStatusRequest, UniversalStatusResponse};
use crate::protocol_details::datatypes::var_types::VarInt;
use crate::protocol_details::protocol_verison::ProtocolVerison;

pub async fn handle_status(connection: &mut CraftClient) -> Result<()> {
	debug!("Handling status for {}", connection);
	
	if connection.peek_next_packet_details().await?.1.0 == 0x01 {
		handle_ping(connection).await?;
		return Ok(());
	}
	
	let status_request = connection.receive_packet::<UniversalStatusRequest>().await?;
	
	trace!("Received status request from {}", connection);
	
	let response = UniversalStatusResponse::new(ProtocolVerison::v1_20_2, "Hello".to_string());
	
	let packed = PackagedPacket::new(VarInt(0x00), response);
	
	connection.send_packet(packed).await?;
	
	trace!("Sent response to {}", connection);
	
	handle_ping(connection).await?;
	
	Ok(())
}

pub async fn handle_ping(connection: &mut CraftClient) -> Result<()> {
	debug!("Handling ping for {}", connection);
	
	let ping_request = connection.receive_packet::<UniversalPingRequest>().await;
	
	trace!("Received ping request from {}", connection);
	
	let packed = PackagedPacket::new(VarInt(0x01), UniversalPingResponse {
		payload: ping_request?.data.payload
	});
	
	connection.send_packet(packed).await?;
	
	trace!("Sent ping to {}", connection);
	
	connection.close().await;
	
	Ok(())
}