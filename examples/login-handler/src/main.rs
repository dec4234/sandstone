use log::{debug, LevelFilter};
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;

use sandstone::network::client::client_handlers::{HandshakeHandler, StatusHandler};
use sandstone::network::client::CraftClient;
use sandstone::protocol::packets::StatusResponsePacket;
use sandstone::protocol::status::{DefaultHandshakeHandler, DefaultPingHandler, DefaultStatusHandler};
use sandstone::protocol::status::status_components::{PlayerSample, StatusResponseSpec};
use sandstone::protocol_types::protocol_verison::ProtocolVerison;


#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Trace).init().unwrap();
    debug!("Starting server");

    let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

    loop {
        let (socket, _) = server.accept().await.unwrap();

        let mut client = CraftClient::from_connection(socket).unwrap();

        
    }
}
