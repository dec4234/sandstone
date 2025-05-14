use log::{debug, LevelFilter};
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;

use sandstone::network::client::client_handlers::{HandshakeHandler, StatusHandler};
use sandstone::network::client::CraftClient;
use sandstone::protocol::packets::StatusResponsePacket;
use sandstone::protocol::status::{DefaultHandshakeHandler, DefaultPingHandler, DefaultStatusHandler};
use sandstone::protocol::status::status_components::{PlayerSample, StatusResponseSpec};
use sandstone::protocol_types::protocol_verison::ProtocolVerison;

/// This demonstrates how to respond to a status request from a client.
/// This returns information used to display the server on the client's server list
/// 
/// After the status has been successfully returned, the connection should terminate. It will
/// be reestablished by the client if it wants to login.
/// 
/// OPTIONAL: Run with --nocapture to see the debug output in real time
/// 
/// The status procedure can be found [here](https://wiki.vg/Server_List_Ping)
#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Trace).init().unwrap();
	debug!("Starting server");

	let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

	loop {
		let (socket, _) = server.accept().await.unwrap();
		
		let mut client = CraftClient::from_connection(socket).unwrap();
		
		let mut response = StatusResponseSpec::new(ProtocolVerison::V1_21, "&a&lThis is a test description &b§kttt");
		response.set_player_info(1, 0, vec![PlayerSample::new_random("&6&lTest")]);
		
		let image = image::open("examples/status_handler/src/server-icon.png").unwrap();
		response.set_favicon_image(image);
		
		DefaultHandshakeHandler::handle_handshake(&mut client).await.unwrap();
		DefaultStatusHandler::handle_status(&mut client, StatusResponsePacket::new(response), DefaultPingHandler).await.unwrap();
	}
}
