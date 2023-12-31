/*use std::net::TcpListener;
use std::time::Duration;
use craftio_rs::{CraftIo, CraftSyncReader, CraftSyncWriter, CraftTcpConnection};
use mcproto_rs::protocol::{HasPacketBody, PacketDirection, State};
use mcproto_rs::{Serializer, SerializeResult};
use mcproto_rs::status::{StatusPlayersSpec, StatusSpec, StatusVersionSpec};
use mcproto_rs::types::Chat;
use mcproto_rs::v1_15_2::{Packet578, RawPacket578, StatusResponseSpec};

#[tokio::test]
async fn simple() {
    let listener = TcpListener::bind("127.0.0.1:25565").unwrap();

    let pair = listener.accept().unwrap();
    let mut stream = pair.0;

    let status = StatusResponseSpec {
        response: StatusSpec {
            description: Chat::from_text("hello"),
            favicon: None,
            players: StatusPlayersSpec {
                max: 10,
                online: 0,
                sample: vec![]
            },
            version: Some(StatusVersionSpec {
                name: "1.19.4".to_string(),
                protocol: 758,
            })
        }
    };

    let mut conn = CraftTcpConnection::from_std(stream, PacketDirection::ServerBound).unwrap();

    let packet2 = conn.read_raw_untyped_packet().unwrap().unwrap();
    println!("{:?}", packet2.1.to_vec());

    conn.set_state(State::Status);

    let packed = Packet578::StatusResponse(status);

    conn.write_packet(packed.clone()).unwrap();

    let vec: Vec<u8> = vec![];

    let mut binding = Some(vec);
    let mut growvec = VecSerializer::create(&mut binding, 0, 1000);

    packed.mc_serialize_body(&mut growvec).unwrap();
    println!("{:?}", binding.unwrap());
}

#[tokio::test]
async fn craftio() {
    let listener = TcpListener::bind("127.0.0.1:25565").unwrap();

    let pair = listener.accept().unwrap();
    let mut stream = pair.0;

    let status = StatusResponseSpec {
        response: StatusSpec {
            description: Chat::from_text("hello"),
            favicon: None,
            players: StatusPlayersSpec {
                max: 10,
                online: 0,
                sample: vec![]
            },
            version: Some(StatusVersionSpec {
                name: "1.19.4".to_string(),
                protocol: 762,
            })
        }
    };

    let mut conn = CraftTcpConnection::from_std(stream, PacketDirection::ServerBound).unwrap();
    //let packet1 = conn.read_packet::<RawPacket578>().unwrap().unwrap();
    let packet2 = conn.read_raw_untyped_packet().unwrap().unwrap();
    println!("{:?}", packet2.1.to_vec());
    conn.set_state(State::Status);

    let packed = Packet578::StatusResponse(status);

    conn.write_packet(packed.clone()).unwrap();

    let vec: Vec<u8> = vec![];

    let mut binding = Some(vec);
    let mut growvec = VecSerializer::create(&mut binding, 0, 1000);

    packed.mc_serialize_body(&mut growvec).unwrap();
    println!("{:?}", binding.unwrap());


}

#[derive(Debug)]
pub struct VecSerializer<'a> {
    target: &'a mut Option<Vec<u8>>,
    at: usize,
    offset: usize,
    max_size: usize,
    exceeded_max_size: bool,
}

impl<'a> Serializer for VecSerializer<'a> {
    fn serialize_bytes(&mut self, data: &[u8]) -> SerializeResult {
        if !self.exceeded_max_size {
            let cur_len = self.written_data_len();
            let new_len = cur_len + data.len();
            if new_len > self.max_size {
                self.exceeded_max_size = true;
            } else {
                get_sized_buf(self.target, self.at + self.offset, data.len()).copy_from_slice(data);
            }
        }

        self.at += data.len();

        Ok(())
    }
}

impl<'a> VecSerializer<'a> {
    pub fn create(target: &'a mut Option<Vec<u8>>, offset: usize, max_size: usize) -> Self {
        Self {
            target,
            at: 0,
            offset,
            max_size,
            exceeded_max_size: false,
        }
    }

    fn written_data_len(&self) -> usize {
        self.at
    }
}

pub fn get_sized_buf(buf: &mut Option<Vec<u8>>, offset: usize, size: usize) -> &mut [u8] {
    let end_at = offset + size;
    loop {
        match buf {
            Some(v) => {
                ensure_buf_has_size(v, end_at);
                break &mut v[offset..end_at];
            }
            None => {
                let new_buf = Vec::with_capacity(end_at);
                *buf = Some(new_buf);
            }
        }
    }
}

fn ensure_buf_has_size(buf: &mut Vec<u8>, total_size: usize) {
    if total_size > buf.len() {
        buf.resize(total_size, 0u8);
    }
}*/