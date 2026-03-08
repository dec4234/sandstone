//! This module defines the network protocol for the server and client.
//! 
//! This includes data types, serializers, packet implementations and client & server handlers.
//! 
//! See the documentation for the [client](client) and [server](server) modules for more information on how to use the network API.

use crate::network::network_error::NetworkError;
use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::packets::Packet;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::protocol_types::protocol_verison::ProtocolVerison;
use log::{debug, error, trace};
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
pub(crate) const CONTINUE_BIT: u8 = 0b10000000;

/// This represents an active Minecraft protocol connection. Either from a client to the server or
/// from the server to a client. This is possible because they are functionally the same.
#[derive(Debug)]
#[allow(dead_code)]
pub struct CraftConnection {
	pub tcp_stream: TcpStream,
	pub socket_addr: SocketAddr,
	pub packet_state: PacketState,
	pub compression_threshold: Option<i32>,
	pub protocol_version: Option<VarInt>,
	pub client_type: PacketDirection,
	/// Reusable buffer for packet reads, avoids allocating per packet
	read_buffer: Vec<u8>,
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
			read_buffer: Vec::with_capacity(1024),
		})
	}

	/// Send a minecraft packet to the client. This will block until the packet is sent.
	pub async fn send_packet(&mut self, packet: Packet) -> Result<(), NetworkError> {
		let mut serializer = McSerializer::new();
		packet.mc_serialize(&mut serializer)?;
		let output = &serializer.output;

		trace!("Sending to {self} : {output:?}");

		// TODO: compress & encrypt here

		self.tcp_stream.write_all(output).await?;
		Ok(())
	}

	/// Receive a minecraft packet from the client. This will block until a packet is received. This removes data from the TCP buffer
	pub async fn receive_packet(&mut self) -> Result<Packet, NetworkError> {
		// Read VarInt length prefix using a stack array (no heap allocation)
		let mut varint_buf = [0u8; 3];
		let mut varint_len = 0usize;

		loop {
			let b = self.tcp_stream.read_u8().await?;
			varint_buf[varint_len] = b;
			varint_len += 1;

			if b & CONTINUE_BIT == 0 {
				break;
			} else if varint_len >= 3 {
				return Err(SerializingErr::VarTypeTooLong("Packet length VarInt max bytes is 3".to_string()).into());
			}
		}

		let packet_len = VarInt::from_slice(&varint_buf[..varint_len])?.0 as usize;

		if packet_len > PACKET_MAX_SIZE {
			return Err(NetworkError::PacketTooLarge);
		}

		// Reuse the connection's read buffer, resizing only when needed
		let total_len = varint_len + packet_len;
		self.read_buffer.resize(total_len, 0);
		self.read_buffer[..varint_len].copy_from_slice(&varint_buf[..varint_len]);

		// Read the full packet body in one call
		match self.tcp_stream.read_exact(&mut self.read_buffer[varint_len..total_len]).await {
			Ok(_) => {}
			Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
				self.close().await;
				return Err(NetworkError::NoDataReceived);
			}
			Err(e) => {
				if e.to_string().contains("An established connection was aborted by the software in your host machine") {
					debug!("OS Error detected in packet receive, closing the connection: {e}");
					self.close().await;
					return Err(NetworkError::ConnectionAbortedLocally);
				}
				return Err(NetworkError::IOError(e));
			}
		}

		trace!("Received from {} : {:?}", self, &self.read_buffer[..total_len]);

		// TODO: decompress & decrypt here

		let mut deserializer = McDeserializer::new(&self.read_buffer[..total_len]);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, self.client_type)?;

		Ok(packet)
	}

	/// Try to receive a packet from the buffer without blocking. This will return 'NoDataReceived'
	/// if no data is available.
	pub fn try_receive_packet(&mut self) -> Result<Packet, NetworkError> {
		let vari = VarInt::from_tcp_stream(&self.tcp_stream)?;
		let (var_buf, var_len) = vari.to_byte_array();

		if vari.0 > PACKET_MAX_SIZE as i32 {
			return Err(NetworkError::PacketTooLarge);
		}

		let packet_len = vari.0 as usize;
		let total_len = var_len + packet_len;
		let mut buffer = vec![0u8; total_len];
		buffer[..var_len].copy_from_slice(&var_buf[..var_len]);

		let length = self.tcp_stream.try_read(&mut buffer[var_len..])?;

		trace!("Received from {} : {:?}", self, &buffer);

		if length == 0 {
			return Err(NetworkError::NoDataReceived);
		} else if length == PACKET_MAX_SIZE {
			return Err(NetworkError::PacketTooLarge);
		}

		// TODO: decompress & decrypt here

		let mut deserializer = McDeserializer::new(&buffer);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, PacketDirection::SERVER)?;

		Ok(packet)
	}

	pub async fn receive_direct<T: McSerialize + McDeserialize>(&mut self) -> Result<T, NetworkError> {
		self.receive_with_length(size_of::<T>()).await
	}

	pub async fn receive_with_length<T: McSerialize + McDeserialize>(&mut self, size: usize) -> Result<T, NetworkError> {
		let mut buffer = vec![0; size];
		let length = self.tcp_stream.read_exact(&mut buffer).await;
		if let Err(e) = length {
			return Err(NetworkError::IOError(e));
		}
		trace!("Received direct from {} : {:?}", self, &buffer);
		let mut deserializer = McDeserializer::new(&buffer);
		let packet = T::mc_deserialize(&mut deserializer)?;
		Ok(packet)
	}

	/// Peek the next packet in the queue without removing it. This will block until a packet is received.
	pub async fn peek_packet(&mut self) -> Result<Packet, NetworkError> {
		// Peek VarInt length using a stack array — we peek incrementally since we
		// don't know how many bytes the VarInt occupies
		let mut peek_buf = [0u8; 3];
		let mut varint_len = 1usize;
		let vari: VarInt;

		loop {
			if self.tcp_stream.peek(&mut peek_buf[..varint_len]).await? == 0 {
				return Err(NetworkError::NoDataReceived);
			}

			if peek_buf[varint_len - 1] & CONTINUE_BIT == 0 {
				vari = VarInt::from_slice(&peek_buf[..varint_len])?;
				break;
			} else if varint_len >= 3 {
				return Err(SerializingErr::VarTypeTooLong("Packet length VarInt max bytes is 3".to_string()).into());
			}

			varint_len += 1;
		}

		if vari.0 > PACKET_MAX_SIZE as i32 {
			return Err(NetworkError::PacketTooLarge);
		}

		let packet_len = vari.0 as usize;
		let total_len = varint_len + packet_len;
		let mut buffer = vec![0u8; total_len];
		buffer[..varint_len].copy_from_slice(&peek_buf[..varint_len]);

		let length = match self.tcp_stream.peek(&mut buffer[varint_len..]).await {
			Ok(len) => len,
			Err(e) => {
				if e.to_string().contains("An established connection was aborted by the software in your host machine") {
					debug!("OS Error detected in packet receive, closing the connection: {e}");
					self.close().await;
					return Err(NetworkError::ConnectionAbortedLocally);
				}
				return Err(NetworkError::IOError(e));
			}
		};

		trace!("Peeked from {} : {:?}", self, &buffer);

		if length == 0 {
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

	/// Peek the next `n` bytes in the queue without removing them. Useful for debugging.
	pub async fn peek_n_bytes(&mut self, n: usize) -> Result<Vec<u8>, NetworkError> {
		let mut buffer = vec![0; n];
		let length = self.tcp_stream.peek(&mut buffer).await;

		if let Err(e) = length {
			if e.to_string().contains("An established connection was aborted by the software in your host machine") {
				error!("OS Error detected in packet receive, closing the connection: {e}");
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
		}

		Ok(buffer)
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
		debug!("Closing connection to {self}");
		self.tcp_stream.shutdown().await.is_ok()
	}

	/// Get the protocol version of this client as a `ProtocolVersion` enum. This will return `None` if the
	/// handshake has not been performed or if the protocol version number is not known to the library
	pub fn get_client_version(&self) -> Option<ProtocolVerison> {
		ProtocolVerison::try_from(self.protocol_version?.0 as i16).ok()
	}
}

impl Display for CraftConnection {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = if let Ok(addr) = self.tcp_stream.peer_addr() {
			format!("{addr}")
		} else {
			"Unknown".to_string()
		};

		write!(f, "{}", format!("CraftConnection: {s}"))
	}
}