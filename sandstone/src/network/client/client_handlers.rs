//! Lists the traits used to handle packet sequences for the client. These are included so that you can
//! override the default functionality for your own purposes.

use crate::network::network_error::NetworkError;
use crate::network::CraftConnection;
use crate::protocol::packets::packet_parts::status::StatusResponseSpec;

/// Lists the methods required to handle a status request as a client. Check [crate::network::client::client_status_handler::DefaultClientStatusHandler] for a default implementation.
///
/// The status procedure can be found [here](https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping#Status_Request)
pub trait ClientStatusHandler {
	async fn handle_status(connection: &mut CraftConnection) -> Result<StatusResponseSpec, NetworkError>;
}
