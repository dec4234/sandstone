use anyhow::Result;
use log::debug;
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;

use crate::network::connection::CraftClient;
use crate::network::network_structure::LoginHandler;

#[tokio::test]
#[ignore]
pub async fn test_status_handler() {
	SimpleLogger::new().init().unwrap();
	debug!("Starting server");

	let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

	loop {
		let (mut socket, a) = server.accept().await.unwrap();
		let mut client = CraftClient::from_connection(socket).unwrap();

		client.handle_handshake(&mut Dummy).await.unwrap();
	}
}

pub struct Dummy;

impl LoginHandler for Dummy {
	fn handle_login(&mut self, _client: &mut CraftClient) -> Result<()> {
		Ok(())
	}
}