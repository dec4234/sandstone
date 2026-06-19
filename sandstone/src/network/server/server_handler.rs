use crate::network::network_error::NetworkError;
use crate::network::CraftConnection;
use crate::protocol::status::status_components::StatusResponseSpec;

/// Lists the methods required to handle a status request as a client. Check [DefaultClientStatusHandler] for a default implementation.
///
/// The status procedure can be found [here](https://wiki.vg/Server_List_Ping)
pub trait ClientStatusHandler {
	async fn handle_status(connection: &mut CraftConnection) -> Result<StatusResponseSpec, NetworkError>;
}
