use log::{debug, LevelFilter};
use simple_logger::SimpleLogger;
use tokio::net::TcpListener;

use sandstone::network::client::client_handlers::{HandshakeHandler, StatusHandler};
use sandstone::network::client::CraftClient;
use sandstone::protocol::packets::packet_definer::{PacketDirection, PacketState};
use sandstone::protocol::packets::{ClientboundKnownPacksPacket, FinishConfigurationPacket, LoginSuccessPacket, Packet, StatusResponsePacket};
use sandstone::protocol::status::{DefaultHandshakeHandler, DefaultPingHandler, DefaultStatusHandler};
use sandstone::protocol::status::status_components::{PlayerSample, StatusResponseSpec};
use sandstone::protocol_types::protocol_verison::ProtocolVerison;
use uuid::Uuid;
use sandstone::protocol::serialization::{McDeserializer, StateBasedDeserializer};
use sandstone::protocol::serialization::serializer_types::PrefixedArray;

#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(LevelFilter::Trace).init().unwrap();
    debug!("Starting server");

    let server = TcpListener::bind("127.0.0.1:25565").await.unwrap();

    let mut response = StatusResponseSpec::new(ProtocolVerison::V1_21, "&a&lThis is a test description &b§kttt");
    response.set_player_info(1, 0, vec![PlayerSample::new_random("&6&lTest")]);

    loop {
        let (socket, _) = server.accept().await.unwrap();

        let mut client = CraftClient::from_connection(socket).unwrap();

        DefaultHandshakeHandler::handle_handshake(&mut client).await.unwrap();

        if client.packet_state == PacketState::STATUS {
            DefaultStatusHandler::handle_status(&mut client, StatusResponsePacket::new(response.clone()), DefaultPingHandler).await.unwrap();
            continue;
        }

        debug!("Beginning login sequence for {}", client);

        let login_start = client.receive_packet().await.unwrap();
        match login_start {
            Packet::LoginStart(packet) => {
                debug!("Received login start packet from {}", client);
                debug!("Login start packet: {:?}", packet);

                // todo: extract player details like username
            }
            _ => {
                debug!("Expected login start packet, got {:?}", login_start);
                continue;
            }
        }
        
        let login_success = Packet::LoginSuccess(LoginSuccessPacket::new(Uuid::new_v4(), "TestUser".to_string(), PrefixedArray::new(vec![])));
        client.send_packet(login_success).await.unwrap();
        
        let login_ack = client.receive_packet().await.unwrap();
        match login_ack {
            Packet::LoginAcknowledged(..) => {
                debug!("Received login acknowledged packet from {}", client);
                debug!("Login acknowledged packet: {:?}", login_ack);
                client.change_state(PacketState::CONFIGURATION);
            }
            _ => {
                debug!("Expected login acknowledged, got {:?}", login_ack);
                continue;
            }
        }
        
        let packs = Packet::ClientboundKnownPacks(ClientboundKnownPacksPacket::new(PrefixedArray::new(vec![])));
        client.send_packet(packs).await.unwrap();
        
        debug!("Sent clientbound known packs to {}", client);
        
        loop {
            let packs = client.receive_packet().await.unwrap();
            match packs {
                Packet::ServerboundPluginMessage(..) => { // optional: before known packs
                    debug!("Received plugin message from {}", client);
                    debug!("Plugin message: {:?}", packs);
                    continue;
                }
                Packet::ServerboundKnownPacks(..) => {
                    debug!("Received clientbound known packs from {}", client);
                    debug!("Clientbound known packs: {:?}", packs);
                    break;
                }
                _ => {
                    debug!("Expected clientbound known packs, got {:?}", packs);
                    break;
                }
            }
        }
        
        let client_info = client.receive_packet().await.unwrap();
        match client_info {
            Packet::ClientInformation(..) => {
                debug!("Received client information from {}", client);
                debug!("Client information: {:?}", client_info);
            }
            _ => {
                debug!("Expected client info, got {:?}", client_info);
            }
        }
        
        let packet = Packet::FinishConfiguration(FinishConfigurationPacket::new());
        client.send_packet(packet).await.unwrap();
        
        debug!("Sent finish configuration to {}", client);
        
        let ack = client.receive_packet().await.unwrap();
        match ack {
            Packet::AcknowledgeFinishConfiguration(..) => {
                debug!("Received acknowledge finish configuration from {}", client);
                debug!("Acknowledge finish configuration: {:?}", ack);
            }
            _ => {
                debug!("Expected acknowledge finish configuration, got {:?}", ack);
                continue;
            }
        }
        
        
    }
}

#[test]
fn test_pack() {
    let bytes = vec![25, 2, 15, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 98, 114, 97, 110, 100, 7, 118, 97, 110, 105, 108, 108, 97];
    let mut deserializer = McDeserializer::new(&bytes);
    let packet = Packet::deserialize_state(&mut deserializer, PacketState::CONFIGURATION, PacketDirection::SERVER).unwrap();
    println!("{:?}", packet);
}
