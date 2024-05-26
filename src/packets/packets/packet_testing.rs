use crate::packets::packet_definer::{PacketDirection, PacketState};
use crate::packets::packets::packet::Packet;
use crate::packets::serialization::serializer_handler::{McDeserializer, StateBasedDeserializer};

#[test]
pub fn test_basic_deserialization() {
	let vec: Vec<u8> = vec![16, 0, 254, 5, 9, 108, 111, 99, 97, 108, 104, 111, 115, 116, 99, 221, 1, 1, 0];
	
	let mut deserializer = McDeserializer::new(&vec);
	let packet = Packet::deserialize_state(&mut deserializer, PacketState::HANDSHAKING, PacketDirection::SERVER).unwrap();
	
	match packet {
		Packet::Handshaking(_) => {}
		_ => panic!("Invalid packet")
	}
	
	let vec: Vec<u8> = vec![9, 1, 0, 0, 0, 0, 0, 26, 36, 46];
	
	let mut deserializer = McDeserializer::new(&vec);
	let packet = Packet::deserialize_state(&mut deserializer, PacketState::STATUS, PacketDirection::SERVER).unwrap();
	
	match packet {
		Packet::PingRequest(_) => {}
		_ => panic!("Invalid packet {:?}", packet)
	}
}