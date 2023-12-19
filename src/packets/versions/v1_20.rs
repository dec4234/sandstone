use std::net::TcpListener;
use craftio_rs::{CraftIo, CraftSyncReader, CraftSyncWriter, CraftTcpConnection, PacketSerializeFail};
use mcproto_rs::protocol::{PacketDirection, State};
use mcproto_rs::{Serializer, SerializeResult};
use mcproto_rs::status::{StatusPlayersSpec, StatusSpec, StatusVersionSpec};
use mcproto_rs::types::Chat;
use mcproto_rs::v1_15_2::{Packet578, RawPacket578, StatusResponseSpec};
use crate::protocol;
use crate::packets::packet_definer::{Packet, PacketState, PacketVersionDefinition};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use uuid::Uuid;
use crate::protocol_details::datatypes::var_types::VarInt;

// https://wiki.vg/Protocol
protocol!(v1_20, 764 => {

    // Server-bound
    StatusRequest, StatusRequestBody, 0x00, STATUS => {
        // none
    },

    // Client bound
    StatusRequestResponse, StatusRequestResponseBody, 0x00, STATUS => {
        version: StatusResponseVersionInfo,
        players: StatusResponsePlayersInfo,
        description: StatusResponseDescriptionInfo,
        favicon: String,
        enforcesSecureChat: bool,
        previewsChat: bool
    },

    PingResponse, PingResponseBody, 0x01, STATUS => {
        payload: u64
    }
});

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusResponseVersionInfo {
    name: String,
    protocol: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusResponsePlayersInfo {
    max: u32,
    online: u32,
    sample: Vec<StatusResponseUserInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusResponseUserInfo {
    name: String,
    id: String, // UUID
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusResponseDescriptionInfo {
    text: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RawPacket {
    Length: VarInt,
    Packet_ID: VarInt,
    Data: Vec<u8>
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{IpAddr, TcpListener};
    use std::str::FromStr;
    use std::thread;
    use std::time::{Duration, Instant};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpSocket;
    use crate::packets::versions::v1_20::{send_status, StatusRequestResponseBody};
    use crate::packets::versions::v1_20::{StatusResponseDescriptionInfo, StatusResponsePlayersInfo, StatusResponseVersionInfo, v1_20};
    use crate::packets::versions::v1_20::v1_20::StatusRequestResponse;
    use crate::protocol_details::datatypes::var_types::VarInt;

    #[test]
    fn basic() {
        println!("Starting...");

        let resp = StatusRequestResponseBody {
            version: StatusResponseVersionInfo {
                name: "1.19.4".to_string(),
                protocol: 762,
            },
            players: StatusResponsePlayersInfo {
                max: 100,
                online: 0,
                sample: vec![],
            },
            description: StatusResponseDescriptionInfo {
                text: "Vindicators 2".to_string(),
            },
            favicon: "".to_string(),
            enforcesSecureChat: true,
            previewsChat: true,
        };

        let mut j = serde_json::to_string(&resp).unwrap();

        //j = "{\"version\": {\"name\": \"1.19.4\",\"protocol\": 762},\"players\": {\"max\": 100,\"online\": 5,\"sample\": [{\"name\": \"thinkofdeath\",\"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"}]},\"description\": {\"text\": \"Hello world\"},\"favicon\": \"\",\"enforcesSecureChat\": true,\"previewsChat\": true}".to_string();

        let vec = j.as_bytes();
        println!("{}", &j);

        let socket = TcpListener::bind("127.0.0.1:25565").unwrap();

        let mut pair = socket.accept().unwrap().0;
        // TODO: read incoming packet data to see format
        let mut v: Vec<u8> = Vec::new();

        pair.set_read_timeout(Some(Duration::from_millis(1000)));

        let size = pair.read_to_end(&mut v);

        v.remove(0);
        v.remove(0);
        v.remove(0);

        v.truncate(12);

        let s = String::from_utf8(v).unwrap();
        println!("Size: , {s}");

        pair.write(VarInt((vec.len() + 1) as i32).to_bytes().as_slice()).unwrap();
        pair.write(VarInt(0).to_bytes().as_slice()).unwrap();
        pair.write(vec).unwrap();
        println!("Connected");
    }

    #[tokio::test]
    async fn test_read() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:25565").await.unwrap();

        let pair = listener.accept().await.unwrap();
        let mut stream = pair.0;

        let mut buf = [0u8; 2048];

        let size = stream.read(&mut buf).await.unwrap();

        let s = String::from_utf8(buf[5..14].to_vec()).unwrap(); // "localhost"
        println!("Received: {s}");

        tokio::time::sleep(Duration::from_millis(300)).await;

        let size = stream.read(&mut buf).await.unwrap();

        println!("Received 2: {:?}", buf[0..size].to_vec());

        tokio::time::sleep(Duration::from_millis(300)).await;

        let mut vec: Vec<u8> = Vec::new();

        for b in VarInt(9).to_bytes() {
            vec.push(b);
        }

        vec.push(0);

        let millis: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        for b in millis.to_le_bytes() {
            vec.push(b);
        }

        let size = stream.write(vec.as_slice()).await.unwrap();
        println!("Wrote Ping: {size}");

        // packet stuff     data           "localhost"                                      25565        status
        // length   id      protocol #     String address                                   port         next state
        // [16,     0,      246, 5,        9, 108, 111, 99, 97, 108, 104, 111, 115, 116,    99, 221,     1]
    }

    #[tokio::test]
    async fn read_all() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:25565").await.unwrap();

        let pair = listener.accept().await.unwrap();
        let mut stream = pair.0;

        let mut buf = [0u8; 2048];

        if let Ok(size) = stream.read(&mut buf).await {
            println!("{:?}", buf[0..size].to_vec());
        }

        send_status(&mut stream).await;
    }
}

async fn send_status(stream: &mut TcpStream) {
    let json = json!({
        "version": {
            "name": "1.19.4",
            "protocol": 762
        },
        "players": {
            "max": 10,
            "online": 0,
            "sample": [

            ]
        },
        "description": {
            "text": "test"
        }
    });

    let j = "{\"version\": {\"name\": \"1.19.4\",\"protocol\": 762},\"players\": {\"max\": 100,\"online\": 5,\"sample\": [{\"name\": \"thinkofdeath\",\"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"}]},\"description\": {\"text\": \"Hello world\"},\"enforcesSecureChat\": true,\"previewsChat\": true}".to_string();
    let jbytes = j.into_bytes();

    let mut out: Vec<u8> = vec![];

    //let jbytes = json.to_string().into_bytes();

    for b in VarInt((jbytes.len() + 1) as i32).to_bytes() {
        out.push(b);
    }

    out.push(0);

    for b in jbytes {
        out.push(b);
    }

    let size = stream.write_all(out.as_slice()).await.unwrap();
    println!("Wrote Status");
}