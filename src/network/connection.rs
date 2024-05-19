use std::fmt::Display;
use std::net::SocketAddr;

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::network::network_structure::LoginHandler;
use crate::packets::packet_definer::PacketState;
use crate::packets::raw_packet::PackagedPacket;
use crate::packets::serialization::serializer_handler::{McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::packets::status::status_handler::handle_status;
use crate::packets::status::status_packets::UniversalHandshakePacket;
use crate::protocol_details::datatypes::var_types::VarInt;

pub struct CraftClient {
	tcp_stream: TcpStream,
	socket_addr: SocketAddr,
	pub packet_state: PacketState
}

impl CraftClient {
	pub fn from_connection(tcp_stream: TcpStream) -> Result<Self> {
		Ok(Self {
			socket_addr: tcp_stream.peer_addr()?,
			tcp_stream,
			packet_state: PacketState::HANDSHAKING
		})
	}
	
	pub async fn send_packet<P: McSerialize + StateBasedDeserializer>(&mut self, packet: PackagedPacket<P>) -> Result<()> {
		let mut serializer = McSerializer::new();
		packet.mc_serialize(&mut serializer)?;
		self.tcp_stream.write_all(&serializer.output).await?;
		Ok(())
	}
	
	pub async fn receive_packet<P: McSerialize + StateBasedDeserializer>(&mut self) -> Result<PackagedPacket<P>> {
		let mut buffer = vec![0; 1024];
		let length = self.tcp_stream.read(&mut buffer).await?;
		let mut deserializer = McDeserializer::new(&buffer[0..length]);
		let packet = PackagedPacket::deserialize_state(&mut deserializer, &self.packet_state)?;
		Ok(packet)
	}
	
	pub fn change_state(&mut self, state: PacketState) {
		self.packet_state = state;
	}
	
	pub async fn handle_handshake<L: LoginHandler>(&mut self, login_handler: &mut L) -> Result<()> {
		if self.packet_state != PacketState::HANDSHAKING {
			return Err(anyhow::anyhow!("Invalid packet state"));
		}
		
		let packet = self.receive_packet::<UniversalHandshakePacket>().await?;
		
		if packet.data.next_state == VarInt(1) {
			self.change_state(PacketState::STATUS);
			handle_status(self).await?;
		} else if packet.data.next_state == VarInt(2) {
			self.change_state(PacketState::LOGIN);
			login_handler.handle_login(self)?;
		} else {
			return Err(anyhow::anyhow!("Invalid next state detected, got \"{}\"", packet.data.next_state.0));
		}
		
		Ok(())
	}
}

impl Display for CraftClient {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = if let Ok(addr) = self.tcp_stream.peer_addr() {
			format!("{}", addr)
		} else {
			"Unknown".to_string()
		};

		write!(f, "{}", format!("CraftConnection: {}", s))
	}
}