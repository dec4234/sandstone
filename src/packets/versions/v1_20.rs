use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::packets::packet_definer::{PacketState, PacketTrait, PacketVersionDefinition};
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::packets::serialization::serializer_handler::DeserializeResult;
use crate::protocol;
use crate::protocol_details::datatypes::var_types::VarInt;

// https://wiki.vg/Protocol
protocol!(v1_20, 764 => {

    // Server-bound
    StatusRequest, StatusRequestBody, 0x00, STATUS => {
        // none
    },

    Handshaking, HandshakingBody, 0x00, HANDSHAKING => {
        protocol_version: VarInt,
        server_address: String,
        port: u16,
        next_state: VarInt
    },

    // Client bound
    /*StatusResponse, StatusResponseBody, 0x00, STATUS => {
        data: StatusSpec
    },*/

    PingResponse, PingResponseBody, 0x01, STATUS => {
        payload: u64
    }
});

#[derive(Debug)]
pub struct RawPacket {
	Length: VarInt,
	Packet_ID: VarInt,
	Data: Vec<u8>
}

#[cfg(test)]
mod tests {
	use tokio::io::AsyncReadExt;

	use crate::packets::raw_packet;
	use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer};
	use crate::packets::versions::v1_20::{send_status, v1_20};

	#[tokio::test]
	#[ignore]
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

	#[tokio::test]
	#[ignore]
	async fn read_handshake() {
		let listener = tokio::net::TcpListener::bind("127.0.0.1:25565").await.unwrap();

		let pair = listener.accept().await.unwrap();
		let mut stream = pair.0;

		let mut buf = [0u8; 2048];

		if let Ok(size) = stream.read(&mut buf).await {
			let mut deserializer = McDeserializer::new(&buf[0..size]);

			let p: raw_packet::PackagedPacket<v1_20> = raw_packet::PackagedPacket::mc_deserialize(&mut deserializer).unwrap();

			match p.data {
				v1_20::StatusRequest(_) => {}
				v1_20::Handshaking(b) => {println!("Address: {}", b.server_address)}
				v1_20::PingResponse(_) => {}
			}
		}

		send_status(&mut stream).await;
	}

	#[test]
	fn try_deserialize() {
		// vari       string                                            u16         vari
		// 251, 5,    9, 108, 111, 99, 97, 108, 104, 111, 115, 116,     99, 221,    1

		let vec: &[u8] = &[16, 0, 251, 5, 9, 108, 111, 99, 97, 108, 104, 111, 115, 116, 99, 221, 1];

		let mut deserializer = McDeserializer::new(vec);

		let p: raw_packet::PackagedPacket<v1_20> = raw_packet::PackagedPacket::mc_deserialize(&mut deserializer).unwrap();

		match p.data {
			v1_20::StatusRequest(_) => {}
			v1_20::Handshaking(b) => {println!("Address: {}", b.server_address)}
			v1_20::PingResponse(_) => {}
		}
	}
}

async fn send_status(stream: &mut TcpStream) {
	let json = json!({
        "version": {
            "name": "1.19.4",
            "protocol": 758
        },
        "players": {
            "max": 10,
            "online": 0,
            "sample": [

            ]
        },
        "description": {
            "text": "test"
        },
        "enforcesSecureChat": true,
        "previewsChat": true
    });

	let j = json.to_string();

	println!("{j}");

	let mut out: Vec<u8> = vec![];

	let jbytes = j.into_bytes();

	for b in VarInt((jbytes.len() + 3) as i32).to_bytes() {
		out.push(b);
	}

	out.push(0);

	for b in VarInt(jbytes.len() as i32).to_bytes() {
		out.push(b);
	}

	for b in jbytes {
		out.push(b);
	}

	let size = stream.write_all(out.as_slice()).await.unwrap();
	println!("Wrote Status");
}