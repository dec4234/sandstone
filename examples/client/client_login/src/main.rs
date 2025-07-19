use log::{debug, error, LevelFilter};
use sandstone::network::CraftConnection;
use sandstone::protocol::packets::{HandshakingPacket, LoginAcknowledgedPacket, Packet};
use sandstone::protocol_types::datatypes::var_types::VarInt;
use simple_logger::SimpleLogger;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();
    debug!("Starting client");

    let socket = TcpStream::connect("127.0.0.1:25565").await.unwrap();

    // Create the client from the socket
    let mut client = CraftConnection::from_connection(socket).unwrap();

    let handshake = Packet::Handshaking(HandshakingPacket {
        protocol_version: VarInt(772),
        server_address: "127.0.0.1".to_string(),
        port: 25565,
        next_state: VarInt(2),
    });

    debug!("Sending handshake packet: {:?}", handshake);
    client.send_packet(handshake).await.unwrap();

    let login_success = client.receive_packet().await.unwrap();

    match login_success {
        Packet::LoginSuccess(packet) => {
            debug!("Login successful: UUID: {}, Username: {}", packet.uuid, packet.username);
        }
        _ => {
            panic!("Unexpected packet received instead of login success: {:?}", login_success);
        }
    }

    let login_ack = Packet::LoginAcknowledged(LoginAcknowledgedPacket {});
    client.send_packet(login_ack).await.unwrap();
    debug!("Sending login acknowledged packet");
}
