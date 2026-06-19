//! Lists the traits used to handle packet sequences for the server. These are included so that you can
//! override the default functionality for your own purposes.

use crate::network::network_error::NetworkError;
use crate::network::CraftConnection;
use crate::protocol::packets::StatusResponsePacket;

/// The procedure required to handle a handshake as a server. Check [DefaultHandshakeHandler] for a default implementation.
///
/// If you would like to implement it yourself then check [here](https://wiki.vg/Protocol#Handshake)
pub trait ServerHandshakeHandler {
	async fn handle_handshake(client: &mut CraftConnection) -> Result<(), NetworkError>;
}

/// Lists the methods required to handle a status request as a server. Check [DefaultStatusHandler] for a default implementation.
///
/// The status procedure can be found [here](https://wiki.vg/Server_List_Ping)
pub trait ServerStatusHandler {
	async fn handle_status<P: ServerPingHandler>(connection: &mut CraftConnection, status_response: StatusResponsePacket, ping_handler: P) -> Result<(), NetworkError>;
}

/// Lists the methods required to handle a ping request as a server. Check [DefaultPingHandler] for a default implementation.
///
/// The ping procedure can be found [here](https://wiki.vg/Server_List_Ping)
pub trait ServerPingHandler {
	async fn handle_ping(connection: &mut CraftConnection) -> Result<(), NetworkError>;
}

/// Lists the methods required to handle a login request as a server. Check [DefaultLoginHandler] for a default implementation.
pub trait ServerLoginHandler {
	fn handle_login(connection: &mut CraftConnection) -> Result<(), NetworkError>;
}
