//! This file is used to define everything relating to a client connection.
//! This includes the connection itself, the ability to send and receive packets, and the ability to
//! change the packet state of the connection.

use std::fmt::Display;
use std::net::SocketAddr;

use log::{debug, trace};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::network::network_error::NetworkError;
use crate::protocol::packet_definer::{PacketDirection, PacketState};
use crate::protocol::packets::Packet;
use crate::protocol::serialization::{McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::protocol_types::protocol_verison::ProtocolVerison;

pub mod client_handlers;

const PACKET_MAX_SIZE: usize = 2097151;  // max of 3 byte VarInt
/// The bit that indicates if a VarInt is continuing into another byte.
const CONTINUE_BIT: u8 = 0b10000000;

/// This represents an active connection to a Minecraft client, from the server's perspective.
/// In other words, this is only created and held from a server context, and does NOT support clients
/// making connections to servers.
#[derive(Debug)]
pub struct CraftClient {
	pub(crate) tcp_stream: TcpStream,
	pub(crate) socket_addr: SocketAddr,
	pub packet_state: PacketState,
	pub compression_threshold: Option<i32>,
	pub client_version: Option<VarInt>
}

impl CraftClient {
	/// Create a new `CraftClient` from a `TcpStream`. This will set the `TcpStream` to use `nodelay` and return an error if it fails to do so.
	pub fn from_connection(tcp_stream: TcpStream) -> Result<Self, NetworkError> {
		tcp_stream.set_nodelay(true)?; // disable Nagle's algorithm - according to WIKI specs

		Ok(Self {
			socket_addr: tcp_stream.peer_addr()?,
			tcp_stream,
			packet_state: PacketState::HANDSHAKING,
			compression_threshold: None,
			client_version: None
		})
	}

	/// Send a minecraft packet to the client. This will block until the packet is sent.
	pub async fn send_packet(&mut self, packet: Packet) -> Result<(), NetworkError> {
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
	pub async fn receive_packet(&mut self) -> Result<Packet, NetworkError> {
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
					return Err(SerializingErr::VarTypeTooLong("Packet length VarInt max bytes is 3".to_string()).into());
				}
			}
		}

		let vari = VarInt::new_from_bytes(vec)?;
		let varbytes = vari.to_bytes();

		if vari.0 > PACKET_MAX_SIZE as i32 { // prob can't happen since it stops after 3 bytes, but check anyways
			return Err(NetworkError::PacketTooLarge);
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
				return Err(NetworkError::ConnectionAbortedLocally);
			}

			return Err(NetworkError::IOError(e));
		}

		let length = length.unwrap();

		trace!("Received from {} : {:?}", self, &buffer);

		if length == 0 { // connection closed
			self.close().await;
			return Err(NetworkError::NoDataReceived);
		} else if length == PACKET_MAX_SIZE {
			return Err(NetworkError::PacketTooLarge);
		}

		// TODO: decompress & decrypt here

		let mut deserializer = McDeserializer::new(&buffer);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, PacketDirection::SERVER)?;

		Ok(packet)
	}
	
	/// Try to receive a packet from the buffer without blocking. This will return 'NoDataReceived' 
	/// if no data is available.
	pub fn try_receive_packet(&mut self) -> Result<Packet, NetworkError> {
		let mut vec = vec![];

		// read varint for length
		loop {
			let var_buffer = &mut [0u8; 1];
			let len = self.tcp_stream.try_read(var_buffer)?;
			
			if len == 0 {
				return Err(NetworkError::NoDataReceived);
			}
			
			let b = var_buffer[0];

			if b & CONTINUE_BIT == 0 {
				vec.push(b);
				break;
			} else {
				vec.push(b);

				if vec.len() > 3 {
					return Err(SerializingErr::VarTypeTooLong("Packet length VarInt max bytes is 3".to_string()).into());
				}
			}
		}

		let vari = VarInt::new_from_bytes(vec)?;
		let varbytes = vari.to_bytes();

		if vari.0 > PACKET_MAX_SIZE as i32 { // prob can't happen since it stops after 3 bytes, but check anyways
			return Err(NetworkError::PacketTooLarge);
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

		let length = self.tcp_stream.try_read(&mut buffer[varbytes.len()..]);

		if let Err(e) = length {
			return Err(NetworkError::IOError(e));
		}

		let length = length.unwrap();

		trace!("Received from {} : {:?}", self, &buffer);

		if length == 0 { // connection closed
			return Err(NetworkError::NoDataReceived);
		} else if length == PACKET_MAX_SIZE {
			return Err(NetworkError::PacketTooLarge);
		}

		// TODO: decompress & decrypt here

		let mut deserializer = McDeserializer::new(&buffer);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, PacketDirection::SERVER)?;

		Ok(packet)
		
	}

	/// Peek the next packet in the queue without removing it. This will block until a packet is received.
	pub async fn peek_packet(&mut self) -> Result<Packet, NetworkError> {
		// read varint for length
		let mut i = 1 as usize;
		let vari: VarInt;

		/*
		So we have to use this weird loop because we need to peek the data slowly to understand the byte length of the varint
		 */
		loop {
			let mut b = vec![0; i];
			if self.tcp_stream.peek(&mut b).await? == 0 {
				return Err(NetworkError::NoDataReceived);
			}

			// this indicates if the varint has ended
			if b[i - 1] & CONTINUE_BIT == 0 {
				vari = VarInt::new_from_bytes(b)?;
				break;
			} else {
				if i > 3 { // any varint over 3 bytes is either broken or too big for a packet
					return Err(SerializingErr::VarTypeTooLong("Packet length VarInt max bytes is 3".to_string()).into());
				}
			}

			i += 1;
		}

		let varbytes = vari.to_bytes();

		if vari.0 > PACKET_MAX_SIZE as i32 { // prob can't happen since it stops after 3 bytes, but check anyways
			return Err(NetworkError::PacketTooLarge);
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
				return Err(NetworkError::ConnectionAbortedLocally);
			}

			return Err(NetworkError::IOError(e));
		}

		let length = length.unwrap();

		trace!("Peeked from {} : {:?}", self, &buffer);

		if length == 0 { // connection closed
			self.close().await;
			return Err(NetworkError::NoDataReceived);
		} else if length == PACKET_MAX_SIZE {
			return Err(NetworkError::PacketTooLarge);
		}

		// TODO: decompress & decrypt here

		let mut deserializer = McDeserializer::new(&buffer);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, PacketDirection::SERVER)?;

		Ok(packet)
	}

	/// Change the internal Packet State. This is used to categorize what kind of packets are being sent/received.
	/// See [PacketState] for more information.
	pub fn change_state(&mut self, state: PacketState) {
		self.packet_state = state;
	}

	/// Enable compression on the connection. This will compress packets that are larger than the threshold.
	pub fn enable_compression(&mut self, threshold: Option<i32>) {
		self.compression_threshold = threshold;
	}

	/// Shutdown the connection as soon as possible
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
