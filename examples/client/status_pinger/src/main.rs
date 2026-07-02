use log::{debug, LevelFilter};
use sandstone::network::client::client_handlers::ClientStatusHandler;
use sandstone::network::client::client_status_handler::DefaultClientStatusHandler;
use sandstone::network::CraftConnection;
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() {
	SimpleLogger::new().with_level(LevelFilter::Debug).init().unwrap();
	debug!("Starting client");

	let mut client = CraftConnection::connect("hypixel.net").await.unwrap();

	let response = DefaultClientStatusHandler::handle_status(&mut client).await.unwrap();

	println!("Server response: {:?}", response);
	println!("Number of players online: {}", response.players.online);
}
