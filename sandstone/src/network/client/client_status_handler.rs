use crate::network::client::client_handlers::ClientStatusHandler;
use crate::network::network_error::NetworkError;
use crate::network::CraftConnection;
use crate::protocol::packets::packet_definer::PacketState;
use crate::protocol::packets::packet_parts::status::StatusResponseSpec;
use crate::protocol::packets::{HandshakingPacket, Packet, StatusRequestPacket};
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::protocol_types::protocol_verison::ProtocolVerison;
use log::debug;

/// Default implementation of the [ClientStatusHandler] trait. This is the default implementation used
/// by the client to handle status requests. It is recommended to use this unless you have a specific reason to override it.
///
/// Use with
/// ```ignore
/// DefaultClientStatusHandler::handle_status(&mut connection).await.unwrap();
/// ```
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