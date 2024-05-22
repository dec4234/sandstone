use std::cmp::PartialEq;
use std::fmt::Display;
use std::net::SocketAddr;

use anyhow::{Error, Result};
use log::{debug, trace};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::network::network_error::{ConnectionAbortedLocally, InvalidPacketState, NoDataReceivedError};
use crate::network::network_structure::LoginHandler;
use crate::packets::packet_definer::PacketState;
use crate::packets::raw_packet::PackagedPacket;
use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::packets::status::status_handler::StatusHandler;
use crate::packets::status::status_packets::UniversalHandshakePacket;
use crate::protocol_details::datatypes::var_types::VarInt;

pub struct CraftClient {
	tcp_stream: TcpStream,
	socket_addr: SocketAddr,
	pub packet_state: PacketState,
	buffer: Vec<u8>
}

impl CraftClient {
	pub fn from_connection(tcp_stream: TcpStream) -> Result<Self> {
		tcp_stream.set_nodelay(true)?;
		
		Ok(Self {
			socket_addr: tcp_stream.peer_addr()?,
			tcp_stream,
			packet_state: PacketState::HANDSHAKING,
			buffer: vec![]
		})
	}
	
	pub async fn send_packet<P: McSerialize + StateBasedDeserializer>(&mut self, packet: PackagedPacket<P>) -> Result<()> {
		let mut serializer = McSerializer::new();
		packet.mc_serialize(&mut serializer)?;
		self.tcp_stream.write_all(&serializer.output).await?;
		Ok(())
	}
	
	pub async fn receive_packet<P: McSerialize + StateBasedDeserializer>(&mut self) -> Result<PackagedPacket<P>> {
		if !self.buffer.is_empty() {
			let mut deserializer = McDeserializer::new(&self.buffer);
			let packet = PackagedPacket::deserialize_state(&mut deserializer, &self.packet_state)?;
			self.buffer = deserializer.collect_remaining().to_vec();
			return Ok(packet);
		}

		/*
		1. if n = buff.len then maybe continuously read until it != len? That should pull all data
		 */
		let mut buffer = vec![0; 1024]; // TODO: what happens if this overflows?
		let length = self.tcp_stream.read(&mut buffer).await;
		
		if let Err(e) = length {
			if e.to_string().contains("An established connection was aborted by the software in your host machine") {
				debug!("OS Error detected in packet receive, closing the connection: {}", e);
				self.close().await;
				return Err(Error::from(ConnectionAbortedLocally));
			}
			
			return Err(Error::from(e));
		}
		
		let length = length.unwrap();
		
		trace!("Received {:?}", &buffer[0..length]);

		if length == 0 { // connection closed
			self.close().await;
			return Err(Error::from(NoDataReceivedError));
		}
		
		let mut deserializer = McDeserializer::new(&buffer[0..length]);
		let packet = PackagedPacket::deserialize_state(&mut deserializer, &self.packet_state)?;

		self.buffer.append(&mut deserializer.collect_remaining().to_vec()); // if the next packet was also collected

		Ok(packet)
	}
	
	pub fn change_state(&mut self, state: PacketState) {
		self.packet_state = state;
	}
	
	pub async fn handle_handshake<L: LoginHandler, S: StatusHandler>(&mut self, status_handler: &S, login_handler: &mut L) -> Result<()> {
		if self.packet_state != PacketState::HANDSHAKING {
			return Err(Error::from(InvalidPacketState));
		}
		
		let packet = self.receive_packet::<UniversalHandshakePacket>().await?;
		
		if packet.data.next_state == VarInt(1) {
			self.change_state(PacketState::STATUS);
			if let Err(e) = status_handler.handle_status(self).await {
				debug!("Error handling status, connection closed: {}", e);
				self.close().await;
				return Ok(())
			}
		} else if packet.data.next_state == VarInt(2) {
			self.change_state(PacketState::LOGIN);
			login_handler.handle_login(self)?;
		} else {
			return Err(anyhow::anyhow!("Invalid next state detected, got \"{}\"", packet.data.next_state.0));
		}
		
		Ok(())
	}
	
	pub async fn peek_next_packet_details(&mut self) -> Result<(VarInt, VarInt)> {
		if !self.buffer.is_empty() {
			let mut deserializer = McDeserializer::new(&self.buffer);
			let length = VarInt::mc_deserialize(&mut deserializer)?;
			let packet_id = VarInt::mc_deserialize(&mut deserializer)?;
			return Ok((length, packet_id));
		}

		let mut buffer = vec![0; 1024];
		let length = self.tcp_stream.peek(&mut buffer).await?;
		
		if length == 0 {
			return Err(anyhow::anyhow!("No data received"));
		}
		
		let mut deserializer = McDeserializer::new(&buffer[0..length]);
		let length = VarInt::mc_deserialize(&mut deserializer)?;
		let packet_id = VarInt::mc_deserialize(&mut deserializer)?;
		Ok((length, packet_id))
	}
	
	pub async fn close(&mut self) -> bool {
		debug!("Closing connection to {}", self);
		self.tcp_stream.shutdown().await.is_ok()
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