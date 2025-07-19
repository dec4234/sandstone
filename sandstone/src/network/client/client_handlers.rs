//! Lists the traits used to handle packet sequences from the client. These are included so that you can
//! override the default functionality for your own purposes.

use crate::network::network_error::NetworkError;
use crate::network::CraftConnection;
use crate::protocol::packets::StatusResponsePacket;

/// The procedure required to handle a handshake. Check [DefaultHandshakeHandler] for a default implementation.
///
/// If you would like to implement it yourself then check [here](https://wiki.vg/Protocol#Handshake)
pub trait HandshakeHandler {
	async fn handle_handshake(client: &mut CraftConnection) -> Result<(), NetworkError>;
}

/// Lists the methods required to handle a status request. Check [DefaultStatusHandler] for a default implementation.
///
/// The status procedure can be found [here](https://wiki.vg/Server_List_Ping)
pub trait StatusHandler {
	async fn handle_status<P: PingHandler>(connection: &mut CraftConnection, status_response: StatusResponsePacket, ping_handler: P) -> Result<(), NetworkError>;
}

/// Lists the methods required to handle a ping request. Check [DefaultPingHandler] for a default implementation.
///
/// The ping procedure can be found [here](https://wiki.vg/Server_List_Ping)
pub trait PingHandler {
	async fn handle_ping(connection: &mut CraftConnection) -> Result<(), NetworkError>;
}

/// Lists the methods required to handle a login request. Check [DefaultLoginHandler] for a default implementation.
pub trait LoginHandler {
	fn handle_login(connection: &mut CraftConnection) -> Result<(), NetworkError>;
}