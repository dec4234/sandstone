use anyhow::Result;
use log::debug;

use crate::network::connection::CraftClient;
use crate::packets::status::status_packets::UniversalStatusRequest;

pub async fn handle_status(connection: &mut CraftClient) -> Result<()> {
	debug!("Handling status for {}", connection);
	
	// TODO: make status packet, send it, and handle the response
	let status_request = connection.receive_packet::<UniversalStatusRequest>().await?;
	
	// TODO: properly record the length of a packet, before and after serialization to put it in a packaged packet
	
	Ok(())
}