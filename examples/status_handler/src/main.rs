use ironcraft::network::connection::{CraftClient, DefaultHandshakeHandler, HandshakeHandler};
use ironcraft::packets::status::status_handler::{DefaultPingHandler, DefaultStatusHandler, StatusHandler};
use ironcraft::packets::status::status_packets::UniversalStatusResponse;
use ironcraft::protocol_details::protocol_verison::ProtocolVerison;
use log::debug;
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();
	debug!("Starting server");

	let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

	loop {
		let (socket, a) = server.accept().await.unwrap();
		
		let mut client = CraftClient::from_connection(socket).unwrap();
		
		let mut response = UniversalStatusResponse::new(ProtocolVerison::v1_20, "§a§lThis is a test description §b§kttt");
		
		let image = image::open("src/server-icon.png").unwrap();
		response.set_favicon_image(image);
		
		DefaultHandshakeHandler::handle_handshake(&mut client).await.unwrap();
		DefaultStatusHandler::handle_status(&mut client, response, DefaultPingHandler).await.unwrap();
	}
}
