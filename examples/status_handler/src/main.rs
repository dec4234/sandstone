use log::debug;
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;

use ironcraft::network::connection::CraftClient;
use ironcraft::packets::status::status_handler::{DefaultHandshakeHandler, DefaultPingHandler, DefaultStatusHandler, HandshakeHandler, StatusHandler};
use ironcraft::packets::status::status_packets::{PlayerSample, UniversalStatusResponse};
use ironcraft::protocol_details::protocol_verison::ProtocolVerison;

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
    SimpleLogger::new().init().unwrap();
	debug!("Starting server");

	let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

	loop {
		let (socket, a) = server.accept().await.unwrap();
		
		let mut client = CraftClient::from_connection(socket).unwrap();
		
		let mut response = UniversalStatusResponse::new(ProtocolVerison::V1_20, "&a&lThis is a test description &b§kttt");
		response.set_player_info(1, 0, vec![PlayerSample::new_random("&6&lTest")]);
		
		let image = image::open("src/server-icon.png").unwrap();
		response.set_favicon_image(image);
		
		DefaultHandshakeHandler::handle_handshake(&mut client).await.unwrap();
		DefaultStatusHandler::handle_status(&mut client, response, DefaultPingHandler).await.unwrap();
	}
}
