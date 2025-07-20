use log::{debug, LevelFilter};
use sandstone::network::CraftConnection;
use sandstone::protocol::packets::packet_definer::{PacketDirection, PacketState};
use sandstone::protocol::packets::{HandshakingPacket, LoginAcknowledgedPacket, LoginStartPacket, Packet, ServerboundKnownPacksPacket};
use sandstone::protocol::serialization::serializer_types::PrefixedArray;
use sandstone::protocol_types::datatypes::var_types::VarInt;
use sandstone::protocol_types::protocol_verison::ProtocolVerison;
use simple_logger::SimpleLogger;
use std::str::FromStr;
use tokio::net::TcpStream;
use uuid::Uuid;

/// This demonstrates the login sequence from a client perspective.
///
/// View the README for more information on how to run this example.
#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();
    debug!("Starting client");

    let socket = TcpStream::connect("127.0.0.1:25565").await.unwrap();

    // Create the client from the socket
    let mut client = CraftConnection::from_connection(socket, PacketDirection::CLIENT).unwrap();

    let handshake = Packet::Handshaking(HandshakingPacket {
        protocol_version: VarInt(ProtocolVerison::most_recent().get_version_number() as i32),
        server_address: "127.0.0.1".to_string(),
        port: 25565,
        next_state: VarInt(2),
    });

    debug!("Sending handshake packet: {:?}", handshake);
    client.send_packet(handshake).await.unwrap();
    
    let login_start = Packet::LoginStart(LoginStartPacket {
        username: "dec4234".to_string(),
        uuid: Uuid::from_str("ef39c197-3c3d-4776-a226-22096378a966").unwrap(),
    });

    debug!("Sending login start packet: {:?}", login_start);
    client.send_packet(login_start).await.unwrap();

    client.change_state(PacketState::LOGIN);

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

    client.change_state(PacketState::CONFIGURATION);

    loop {
        let packet = client.receive_packet().await.unwrap();

        match packet {
            Packet::ClientboundPluginMessage(_) => {
                debug!("Received clientbound plugin message: {:?}", packet);
                continue;
            }
            Packet::FeatureFlags(_) => {
                debug!("Received feature flags: {:?}", packet);
                continue;
            }
            Packet::ClientboundKnownPacks(_) => {
                debug!("Received known packs: {:?}", packet);
                break;
            }
            _ => {
                panic!("Received unexpected packet: {:?}", packet);
            }
        }
    }

    let serverbound_known_packs = Packet::ServerboundKnownPacks(ServerboundKnownPacksPacket {
        entries: PrefixedArray::new(vec![]),
    });

    debug!("Sending serverbound known packs: {:?}", serverbound_known_packs);
    client.send_packet(serverbound_known_packs).await.unwrap();

    loop {
        let packet = client.receive_packet().await.unwrap();

        match packet {
            Packet::RegistryData(pack) => {
                debug!("Received registry data: {:?}", pack);

                continue;
            }
            Packet::UpdateTags(_) => {
                debug!("Received update tags: {:?}", packet);
                continue;
            }
            Packet::FinishConfiguration(_) => {
                debug!("Received finish configuration packet: {:?}", packet);
                break;
            }
            _ => {
                panic!("Received unexpected packet: {:?}", packet);
            }
        }
    }

    //client.change_state(PacketState::PLAY);

    // todo: registry data right after
}
