use std::fmt::Display;
use std::net::SocketAddr;

use anyhow::{Error, Result};
use log::{debug, trace};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::network::network_error::{ConnectionAbortedLocally, NoDataReceivedError};
use crate::packets::packet_definer::{PacketDirection, PacketState};
use crate::packets::packets::packet::Packet;
use crate::packets::serialization::serializer_handler::{McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol_details::datatypes::var_types::VarInt;
use crate::protocol_details::protocol_verison::ProtocolVerison;

const PACKET_MAX_SIZE: usize = 2097151; // max of 3 byte VarInt
const CONTINUE_BIT: u8 = 0b10000000;

pub struct CraftClient {
	tcp_stream: TcpStream,
	socket_addr: SocketAddr,
	pub packet_state: PacketState,
	compression_threshold: Option<i32>,
	buffer: Vec<u8>,
	client_version: Option<VarInt>
}

impl CraftClient {
	pub fn from_connection(tcp_stream: TcpStream) -> Result<Self> {
		tcp_stream.set_nodelay(true)?; // disable Nagle's algorithm

		Ok(Self {
			socket_addr: tcp_stream.peer_addr()?,
			tcp_stream,
			packet_state: PacketState::HANDSHAKING,
			compression_threshold: None,
			buffer: vec![],
			client_version: None
		})
	}

	pub async fn send_packet(&mut self, packet: Packet) -> Result<()> {
		let mut serializer = McSerializer::new();
		packet.mc_serialize(&mut serializer)?;
		let output = &serializer.output;

		trace!("Sending to {} : {:?}", self, output);

		// TODO: compress & encrypt here

		self.tcp_stream.write_all(output).await?;
		Ok(())
	}

	// TODO: could use a good optimization pass - reduce # of copies, ideally to 0
	/// Receive a minecraft packet from the client. This will block until a packet is received. This removes data from the TCP buffer
	pub async fn receive_packet(&mut self) -> Result<Packet> {
		let mut vec = vec![];

		// read varint for length
		loop {
			let b = self.tcp_stream.read_u8().await?;

			if b & CONTINUE_BIT == 0 {
				vec.push(b);
				break;
			} else {
				vec.push(b);

				if vec.len() > 3 {
					return Err(anyhow::anyhow!("VarInt too long"));
				}
			}
		}

		let vari = VarInt::new_from_bytes(vec)?;
		let varbytes = vari.to_bytes();

		if vari.0 > PACKET_MAX_SIZE as i32 { // prob can't happen since it stops after 3 bytes, but check anyways
			return Err(anyhow::anyhow!("Packet too large"));
		}

		let length = vari.0 as usize + varbytes.len();

		// TODO: analysis needed - does this minimize copying?
		// could define &[u8] to max packet size but that seems like too much memory usage
		let mut buffer = vec![0; length];

		let mut i = 0;

		for b in &varbytes {
			buffer[i] = *b;
			i += 1;
		}

		let length = self.tcp_stream.read(&mut buffer[varbytes.len()..]).await;

		if let Err(e) = length {
			if e.to_string().contains("An established connection was aborted by the software in your host machine") {
				debug!("OS Error detected in packet receive, closing the connection: {}", e);
				self.close().await;
				return Err(Error::from(ConnectionAbortedLocally));
			}

			return Err(Error::from(e));
		}

		let length = length.unwrap();

		trace!("Received from {} : {:?}", self, &buffer);

		if length == 0 { // connection closed
			self.close().await;
			return Err(Error::from(NoDataReceivedError));
		} else if length == PACKET_MAX_SIZE {
			return Err(anyhow::anyhow!("Packet too large"));
		}

		// TODO: decompress & decrypt here

		let mut deserializer = McDeserializer::new(&buffer);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, PacketDirection::SERVER)?;

		Ok(packet)
	}

	pub async fn peek_packet(&mut self) -> Result<Packet> {
		// read varint for length
		let mut i = 1 as usize;
		let vari: VarInt;

		/*
		So we have to use this weird loop because we need to peek the data slowly to understand the byte length of the varint
		 */
		loop {
			let mut b = vec![0; i];
			if self.tcp_stream.peek(&mut b).await? == 0 {
				return Err(Error::from(NoDataReceivedError));
			}
			
			// this indicates if the varint has ended
			if b[i - 1] & CONTINUE_BIT == 0 {
				vari = VarInt::new_from_bytes(b)?;
				break;
			} else {
				if i > 3 { // any varint over 3 bytes is either broken or too big for a packet
					return Err(anyhow::anyhow!("VarInt too long"));
				}
			}
			
			i += 1;
		}
		
		let varbytes = vari.to_bytes();

		if vari.0 > PACKET_MAX_SIZE as i32 { // prob can't happen since it stops after 3 bytes, but check anyways
			return Err(anyhow::anyhow!("Packet too large"));
		}

		let length = vari.0 as usize + varbytes.len();

		// TODO: analysis needed - does this minimize copying?
		// could define &[u8] to max packet size but that seems like too much memory usage
		let mut buffer = vec![0; length];

		let mut i = 0;

		for b in &varbytes {
			buffer[i] = *b;
			i += 1;
		}

		let length = self.tcp_stream.peek(&mut buffer[varbytes.len()..]).await;

		if let Err(e) = length {
			if e.to_string().contains("An established connection was aborted by the software in your host machine") {
				debug!("OS Error detected in packet receive, closing the connection: {}", e);
				self.close().await;
				return Err(Error::from(ConnectionAbortedLocally));
			}

			return Err(Error::from(e));
		}

		let length = length.unwrap();

		trace!("Peeked from {} : {:?}", self, &buffer);

		if length == 0 { // connection closed
			self.close().await;
			return Err(Error::from(NoDataReceivedError));
		} else if length == PACKET_MAX_SIZE {
			return Err(anyhow::anyhow!("Packet too large"));
		}

		// TODO: decompress & decrypt here

		let mut deserializer = McDeserializer::new(&buffer);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, PacketDirection::SERVER)?;

		Ok(packet)
	}

	pub fn change_state(&mut self, state: PacketState) {
		self.packet_state = state;
	}

	pub fn enable_compression(&mut self, threshold: Option<i32>) {
		self.compression_threshold = threshold;
	}

	pub async fn close(&mut self) -> bool {
		debug!("Closing connection to {}", self);
		self.tcp_stream.shutdown().await.is_ok()
	}

	/// Get the protocol version of this client as a `ProtocolVersion` enum. This will return 'None' if the 
	/// handshake has not been performed or if the protocol version number is not known to the library
	pub fn get_client_version(&self) -> Option<ProtocolVerison> {
		Some(ProtocolVerison::from(self.client_version?.0 as i16)?)
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