use crate::packets::packet_definer::{PacketDirection, PacketState};
use crate::packets::packets::packet::{LoginPluginResponseBody, Packet};
use crate::packets::packets::packet_component::LoginPluginSpec;
use crate::packets::serialization::serializer_handler::{McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};

#[test]
pub fn test_basic_deserialization() {
	let vec: Vec<u8> = vec![16, 0, 254, 5, 9, 108, 111, 99, 97, 108, 104, 111, 115, 116, 99, 221, 1, 1, 0]; // Handshake
	
	let mut deserializer = McDeserializer::new(&vec);
	let packet = Packet::deserialize_state(&mut deserializer, PacketState::HANDSHAKING, PacketDirection::SERVER).unwrap();
	
	match packet {
		Packet::Handshaking(_) => {}
		_ => panic!("Invalid packet")
	}
	
	let vec: Vec<u8> = vec![9, 1, 0, 0, 0, 0, 0, 26, 36, 46]; // PingRequest
	
	let mut deserializer = McDeserializer::new(&vec);
	let packet = Packet::deserialize_state(&mut deserializer, PacketState::STATUS, PacketDirection::SERVER).unwrap();
	
	match packet {
		Packet::PingRequest(_) => {}
		_ => panic!("Invalid packet {:?}", packet)
	}
}

#[test]
pub fn test_optional_vec_serialization() {
	let mut serializer = McSerializer::new();
	
	let packet = Packet::LoginPluginResponse(LoginPluginResponseBody {
		response: LoginPluginSpec {
			message_id: 0.into(),
			success: true,
			data: Some(vec![1, 2, 3])
		}
	});
	
	packet.mc_serialize(&mut serializer).unwrap();
	
	let output = &serializer.output;
	
	let mut deserializer = McDeserializer::new(output);
	let out = Packet::deserialize_state(&mut deserializer, PacketState::LOGIN, PacketDirection::SERVER).unwrap();
	assert_eq!(packet, out);
	
	serializer.clear();
	
	let packet = Packet::LoginPluginResponse(LoginPluginResponseBody {
		response: LoginPluginSpec {
			message_id: 0.into(),
			success: false,
			data: None
		}
	});
	
	packet.mc_serialize(&mut serializer).unwrap();
	
	let output = &serializer.output;
	
	let mut deserializer = McDeserializer::new(output);
	let out = Packet::deserialize_state(&mut deserializer, PacketState::LOGIN, PacketDirection::SERVER).unwrap();
	assert_eq!(packet, out);
}