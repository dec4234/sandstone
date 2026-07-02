//! Lists the traits used to handle packet sequences for the server. These are included so that you can
//! override the default functionality for your own purposes.

use crate::network::network_error::NetworkError;
use crate::network::CraftConnection;
use crate::protocol::packets::StatusResponsePacket;
use mojang_api::HasJoinedResponse;

/// The procedure required to handle a handshake as a server. Check [DefaultHandshakeHandler] for a default implementation.
///
/// If you would like to implement it yourself then check [here](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Handshake)
pub trait ServerHandshakeHandler {
	async fn handle_handshake(connection: &mut CraftConnection) -> Result<(), NetworkError>;
}

/// Lists the methods required to handle a status request as a server. Check [DefaultClientStatusHandler] for a default implementation.
///
/// The status procedure can be found [here](https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping#Status_Request)
pub trait ServerStatusHandler {
	async fn handle_status<P: ServerPingHandler>(connection: &mut CraftConnection, status_response: StatusResponsePacket, ping_handler: P) -> Result<(), NetworkError>;
}

/// Lists the methods required to handle a ping request as a server. Check [DefaultPingHandler] for a default implementation.
///
/// The ping procedure can be found [here](https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping#Ping_Request)
pub trait ServerPingHandler {
	async fn handle_ping(connection: &mut CraftConnection) -> Result<(), NetworkError>;
}

/// Lists the methods required to handle a login request as a server. Check [DefaultLoginHandler] for a default implementation.
pub trait ServerLoginHandler {
	async fn handle_login(connection: &mut CraftConnection) -> Result<(), NetworkError>;
}

/// Handle enabling encryption for an incoming client connections.
pub trait ServerEncryptionHandler {
	/// ## Arguments
	/// * `connection` - The connection to the client.
	/// * `should_authenticate_with_mojang` - Whether to authenticate with Mojang. If false, the server will skip authentication and allow any username.
	/// * `username` - The username of the client. Must match what was sent by the client in the login start packet.
	/// * `server_ip` - The IP address of the server. If included, it should block connections from proxies. Typically not included.
	async fn handle_encryption(connection: &mut CraftConnection, should_authenticate_with_mojang: bool, username: String, server_ip: Option<String>)
	-> Result<Option<HasJoinedResponse>, NetworkError>;
}
