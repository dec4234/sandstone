//! Benchmarking for the protocol module.

use std::time::SystemTime;

use crate::protocol::packet_definer::{PacketDirection, PacketState};
use crate::protocol::packets::{HandshakingBody, Packet};
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol_types::datatypes::var_types::VarInt;

#[ignore]
#[test]
fn benchmark_mass_packet_serializations() {
	const ITERATIONS: usize = 100000;
	
	let current_millis = SystemTime::now();
	
	let mut serializer = McSerializer::init_size(10000000);
	
	let packet = Packet::Handshaking(HandshakingBody {
		protocol_version: VarInt(754),
		server_address: "localhost".to_string(),
		port: 25565,
		next_state: VarInt(1),
	});
	
	for _ in 0..ITERATIONS {
		packet.mc_serialize(&mut serializer).unwrap();
	}
	
	println!("Average time taken to serialize {ITERATIONS} packets: {:.3}micros", SystemTime::now().duration_since(current_millis).unwrap().as_micros() as f64 / ITERATIONS as f64);
	
	let current_millis = SystemTime::now();
	
	let mut deserializer = McDeserializer::new(&serializer.output);
	
	for _ in 0..ITERATIONS {
		let _ = Packet::deserialize_state(&mut deserializer, PacketState::HANDSHAKING, PacketDirection::SERVER).unwrap();
	}
	
	println!("Average time taken to deserialize {ITERATIONS} packets: {:.3}micros", SystemTime::now().duration_since(current_millis).unwrap().as_micros() as f64 / ITERATIONS as f64);
}

#[ignore]
#[test]
fn benchmark_mass_varint_serialization() {
	const ITERATIONS: usize = 100000;
	
	let current_millis = SystemTime::now();
	
	let mut serializer = McSerializer::init_size(1000000);
	
	for i in 0..ITERATIONS {
		VarInt(i as i32).mc_serialize(&mut serializer).unwrap();
	}
	
	println!("Average time taken to serialize {ITERATIONS} varints: {:.3}micros", SystemTime::now().duration_since(current_millis).unwrap().as_micros() as f64 / ITERATIONS as f64);
	
	let current_millis = SystemTime::now();
	
	let mut deserializer = McDeserializer::new(&serializer.output);
	
	for _ in 0..ITERATIONS {
		let _ = VarInt::mc_deserialize(&mut deserializer).unwrap();
	}
	
	println!("Average time taken to deserialize {ITERATIONS} varints: {:.3}micros", SystemTime::now().duration_since(current_millis).unwrap().as_micros() as f64 / ITERATIONS as f64);
}

#[ignore]
#[test]
pub fn benchmark_get_bytes_of_varint() {
	const ITERATIONS: usize = 10000000;
	
	let current_millis = SystemTime::now();
	
	for i in 0..ITERATIONS {
		let _ = VarInt(i as i32).to_bytes();;
	}
	
	println!("Average time taken to get bytes of {ITERATIONS} VarInts: {:.3}micros", SystemTime::now().duration_since(current_millis).unwrap().as_micros() as f64 / ITERATIONS as f64);
}