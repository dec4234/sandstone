use anyhow::Result;
use log::debug;
use simple_logger::SimpleLogger;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

use crate::network::connection::{CraftClient, DefaultHandshakeHandler, HandshakeHandler};
use crate::network::network_structure::LoginHandler;
use crate::packets::status::status_handler::{DefaultPingHandler, DefaultStatusHandler, StatusHandler};
use crate::packets::status::status_packets::UniversalStatusResponse;
use crate::protocol_details::protocol_verison::ProtocolVerison;

#[tokio::test]
#[ignore]
pub async fn test_status_handler() {
	SimpleLogger::new().init().unwrap();
	debug!("Starting server");

	let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

	loop {
		let (socket, a) = server.accept().await.unwrap();
		
		let mut client = CraftClient::from_connection(socket).unwrap();
		
		let mut response = UniversalStatusResponse::new(ProtocolVerison::v1_20, "§a§lThis is a test description §b§kttt");
		
		let image = image::open("test/resources/server-icon.png").unwrap();
		response.set_favicon_image(image);
		
		DefaultHandshakeHandler::handle_handshake(&mut client).await.unwrap();
		DefaultStatusHandler::handle_status(&mut client, response, DefaultPingHandler).await.unwrap();
	}
}

pub struct Dummy;

impl LoginHandler for Dummy {
	fn handle_login(client: &mut CraftClient) -> Result<()> {
		Ok(())
	}
}