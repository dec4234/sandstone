//! This module defines the network protocol for the server and client.
//! 
//! This includes data types, serializers, packet implementations and client & server handlers.
//! 
//! See the documentation for the [client](client) and [server](server) modules for more information on how to use the network API.

use crate::network::network_error::NetworkError;
use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::packets::Packet;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::protocol_types::protocol_verison::ProtocolVerison;
use log::{debug, trace};
use std::fmt::Display;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub mod network_error;
pub mod client;
pub mod server;

/// The maximum size of a packet in bytes.
const PACKET_MAX_SIZE: usize = 2097151;
// max of 3 byte VarInt
/// The bit that indicates if a VarInt is continuing into another byte.
const CONTINUE_BIT: u8 = 0b10000000;

/// This represents an active Minecraft protocol connection. Either from a client to the server or
/// from the server to a client. This is possible because they are functionally the same.
#[derive(Debug)]
#[allow(dead_code)]
pub struct CraftConnection {
	pub(crate) tcp_stream: TcpStream,
	pub(crate) socket_addr: SocketAddr,
	pub packet_state: PacketState,
	pub compression_threshold: Option<i32>,
	pub protocol_version: Option<VarInt>,
	pub client_type: PacketDirection,
}

impl CraftConnection {
	/// Create a new `CraftClient` from a `TcpStream`. This will set the `TcpStream` to use `nodelay` and return an error if it fails to do so.
	/// 
	/// Set client_type to `PacketDirection::CLIENT` if this is a client, or `PacketDirection::SERVER` if this is a server's connection to a client.
	pub fn from_connection(tcp_stream: TcpStream, client_type: PacketDirection) -> Result<Self, NetworkError> {
		tcp_stream.set_nodelay(true)?; // disable Nagle's algorithm - according to WIKI specs

		Ok(Self {
			socket_addr: tcp_stream.peer_addr()?,
			tcp_stream,
			packet_state: PacketState::HANDSHAKING,
			compression_threshold: None,
			protocol_version: None,
			client_type,
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
		let mut vec = Vec::with_capacity(3);

		// read varint for length
		loop {
			let b = self.tcp_stream.read_u8().await?;

			vec.push(b);

			if b & CONTINUE_BIT == 0 {
				break;
			} else if vec.len() > 3 {
				return Err(SerializingErr::VarTypeTooLong("Packet length VarInt max bytes is 3".to_string()).into());
			}
		}

		let vari = VarInt::from_slice(&vec)?;

		if vari.0 > PACKET_MAX_SIZE as i32 { // prob can't happen since it stops after 3 bytes, but check anyways
			return Err(NetworkError::PacketTooLarge);
		}

		let length = vari.0 as usize + vec.len();

		// TODO: analysis needed - does this minimize copying?
		// could define &[u8] to max packet size but that seems like too much memory usage
		let mut buffer = vec![0; length];

		let mut i = 0;

		for b in &vec {
			buffer[i] = *b;
			i += 1;
		}

		let length = self.tcp_stream.read(&mut buffer[vec.len()..]).await;

		let length = match length {
			Ok(length) => {length}
			Err(e) => {
				if e.to_string().contains("An established connection was aborted by the software in your host machine") {
					debug!("OS Error detected in packet receive, closing the connection: {}", e);
					self.close().await;
					return Err(NetworkError::ConnectionAbortedLocally);
				}

				return Err(NetworkError::IOError(e));
			}
		};

		trace!("Received from {} : {:?}", self, &buffer);

		if length == 0 { // connection closed
			self.close().await;
			return Err(NetworkError::NoDataReceived);
		} else if length == PACKET_MAX_SIZE {
			return Err(NetworkError::PacketTooLarge);
		}

		// TODO: decompress & decrypt here

		let mut deserializer = McDeserializer::new(&buffer);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, self.client_type)?;

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

		let vari = VarInt::from_slice(&vec)?;
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

		let length = length?;

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
		let mut i = 1usize;
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
				vari = VarInt::from_slice(&b)?;
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

		let length = length?;

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

	/// Get the protocol version of this client as a `ProtocolVersion` enum. This will return `None` if the
	/// handshake has not been performed or if the protocol version number is not known to the library
	pub fn get_client_version(&self) -> Option<ProtocolVerison> {
		Some(ProtocolVerison::from(self.protocol_version?.0 as i16)?)
	}
}

impl Display for CraftConnection {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = if let Ok(addr) = self.tcp_stream.peer_addr() {
			format!("{}", addr)
		} else {
			"Unknown".to_string()
		};

		write!(f, "{}", format!("CraftConnection: {}", s))
	}
}