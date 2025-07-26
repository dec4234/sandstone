//! Collect registry data and save it as JSON files for development purposes.

use log::{debug, error, LevelFilter};
use sandstone::network::CraftConnection;
use sandstone::protocol::packets::packet_definer::{PacketDirection, PacketState};
use sandstone::protocol::packets::{HandshakingPacket, LoginAcknowledgedPacket, LoginStartPacket, Packet, ServerboundKnownPacksPacket};
use sandstone::protocol::serialization::serializer_error::SerializingErr;
use sandstone::protocol::serialization::serializer_types::{PrefixedArray, PrefixedOptional};
use sandstone::protocol::serialization::McDeserialize;
use sandstone::protocol::serialization::McDeserializer;
use sandstone::protocol::serialization::McSerialize;
use sandstone::protocol::serialization::McSerializer;
use sandstone::protocol::serialization::SerializingResult;
use sandstone::protocol_types::datatypes::nbt::nbt::NbtCompound;
use sandstone::protocol_types::datatypes::var_types::VarInt;
use sandstone::protocol_types::protocol_verison::ProtocolVerison;
use sandstone_derive::{McDeserialize, McSerialize};
use serde::ser::SerializeStruct;
use serde::Serialize;
use simple_logger::SimpleLogger;
use std::fs;
use std::str::FromStr;
use std::time::Duration;
use tokio::net::TcpStream;
use uuid::Uuid;

/// This collects registry data sent by the server to the client during the login process and saves
/// it as JSON files in the `reg_data` directory.
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

    debug!("Current dir is: {}", std::env::current_dir().unwrap().display());

    let mut i = 0;

    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let length = VarInt::from_tcp_stream(&client.tcp_stream).unwrap();

        match client.receive_with_length::<RawPacket<RegPacket>>(length.0 as usize).await {
            Ok(raw) => {
                let regpacket = raw.data;

                let id = regpacket.id.clone().replace("minecraft:", "").replace("/", "_");

                // Ensure the output directory exists
                let output_dir = "reg_data";
                fs::create_dir_all(output_dir).unwrap();

                // Create a unique filename, e.g., using the regpacket id
                let filename = format!("{}/{}.json", output_dir, id);

                // Serialize to JSON and write to file
                let json = serde_json::to_string_pretty(&regpacket).unwrap();
                debug!("Saved raw packet for {} to {}", id, filename);
                fs::write(&filename, json).unwrap();
            }
            Err(e) => {
                error!("Failed to receive raw packet: {:?}", e);
            }
        }

        i += 1;
    }
}

#[derive(Debug)]
pub struct RawPacket<T: McDeserialize + McSerialize> {
    pub id: VarInt,
    pub data: T
}

impl<T: McDeserialize + McSerialize> McDeserialize for RawPacket<T> {
    fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
    where
        Self: Sized
    {
        Ok(Self {
            id: VarInt::mc_deserialize(deserializer)?,
            data: T::mc_deserialize(deserializer)?
        })
    }
}

impl<T: McDeserialize + McSerialize> McSerialize for RawPacket<T> {
    fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
        self.id.mc_serialize(serializer)?;
        self.data.mc_serialize(serializer)?;
        Ok(())
    }
}

#[derive(Debug, McDeserialize, McSerialize)]
pub struct RegPacket {
    pub id: String,
    pub entries: PrefixedArray<Entry>
}

impl Serialize for RegPacket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut state = serializer.serialize_struct("RegPacket", 2)?;
        state.serialize_field("id", &self.id)?;

        // Use slice() to get the actual entries for serialization
        state.serialize_field("entries", self.entries.slice())?;

        state.end()
    }
}

#[derive(Debug, McDeserialize, McSerialize)]
pub struct Entry {
    pub identifier: String,
    pub data: PrefixedOptional<NbtCompound>
}

impl Serialize for Entry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        let mut state = serializer.serialize_struct("Entry", 2)?;
        state.serialize_field("identifier", &self.identifier)?;

        if self.data.is_present() {
            state.serialize_field("data", &self.data.value().unwrap())?;
        }
        state.end()
    }
}
