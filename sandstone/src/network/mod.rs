//! This module defines the network protocol for the server and client.
//!
//! This includes data types, serializers, packet implementations and client & server handlers.
//!
//! See the documentation for the [client](client) and [server](server) modules for more information on how to use the network API.

use crate::network::network_error::NetworkError;
use crate::protocol::packets::Packet;
use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::protocol_types::protocol_verison::ProtocolVerison;
use crate::util::encryption::aes::CraftCipher;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use hickory_resolver::Resolver;
use hickory_resolver::proto::rr::RecordRef;
use hickory_resolver::proto::rr::rdata::SRV;
use log::{debug, error, trace};
use std::fmt::Display;
use std::io::{Read, Write};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub mod client;
pub mod network_error;
pub mod server;

/// A type that is an alias for [PacketDirection] to prevent some naming confusion for connection intialization.
pub type ConnectionRole = PacketDirection;

/// Default minecraft server port.
pub const DEFAULT_PORT: u16 = 25565;
/// The maximum size of a packet in bytes.
const PACKET_MAX_SIZE: usize = 2097151;
// max of 3 byte VarInt
/// The bit that indicates if a VarInt is continuing into another byte.
pub(crate) const CONTINUE_BIT: u8 = 0b10000000;

/// This represents an active Minecraft protocol connection. Either from a client to the server or
/// from the server to a client. This is possible because they are functionally the same.
#[derive(Debug)]
pub struct CraftConnection {
	pub tcp_stream: TcpStream,
	pub socket_addr: SocketAddr,
	/// The hostname the client dialed (e.g. `hypixel.net`), if known. Used as the handshake
	/// `server_address` so proxied servers (BungeeCord/Velocity) can route the connection.
	/// Falls back to the peer IP when `None`.
	pub hostname: Option<String>,
	pub packet_state: PacketState,
	pub compression_threshold: Option<u32>,
	pub protocol_version: Option<VarInt>,
	pub client_type: ConnectionRole,
	craft_cipher: Option<CraftCipher>,
	/// Reusable buffer for packet reads, avoids allocating per packet
	read_buffer: Vec<u8>,
	/// Bytes of a frame that [CraftConnection::try_receive_packet] has received but not yet decoded
	pending_frame: Vec<u8>,
}

impl CraftConnection {
	/// Dial a server and create a client `CraftConnection`. The `address` (e.g. `hypixel.net`)
	/// is used both to open the socket and is stored as the handshake `server_address`, so proxied
	/// servers (BungeeCord/Velocity) can route the connection. Use this for all client connections.
	///
	/// Sets client_type to [ConnectionRole::CLIENT] since this is a client connection to a server.
	pub async fn connect(address: impl AsRef<str>) -> Result<Self, NetworkError> {
		let address = address.as_ref();
		// the handshake server_address is the host without the port. When an explicit port is given
		// we dial it directly; otherwise we resolve the Minecraft SRV record (matching the Notchian
		// client) and fall back to the default port when no SRV record exists.
		let (host, dial) = match address.rsplit_once(':') {
			Some((host, _)) => (host.to_string(), address.to_string()),
			None => {
				let host = address.to_string();
				let dial = Self::resolve_srv(&host).await.unwrap_or_else(|| format!("{host}:{DEFAULT_PORT}"));
				(host, dial)
			}
		};
		let tcp_stream = TcpStream::connect(dial).await?;

		let mut connection = Self::from_connection(tcp_stream, ConnectionRole::CLIENT)?;
		// keep the originally dialed hostname for the handshake so proxied servers route correctly
		connection.hostname = Some(host);

		Ok(connection)
	}

	/// Resolve the Minecraft SRV record (`_minecraft._tcp.<host>`) for `host`, returning a
	/// `target:port` dial string when a record exists. Minecraft servers commonly publish an SRV
	/// record so the apex domain (e.g. `2b2t.org`) can point its A record at a website while the
	/// actual server lives elsewhere; without this lookup such hostnames connect to the wrong
	/// endpoint and hang. Returns `None` (so the caller falls back to `host:25565`) on any DNS
	/// failure or when no record is published.
	async fn resolve_srv(host: &str) -> Option<String> {
		let resolver = Resolver::builder_tokio().ok()?.build().ok()?;
		let lookup = resolver.srv_lookup(format!("_minecraft._tcp.{host}.")).await.ok()?;
		let srv = lookup.answers().iter().find_map(|r| RecordRef::<SRV>::try_from(r).ok())?;
		let srv = srv.data();
		let target = srv.target.to_utf8();
		let target = target.trim_end_matches('.');
		Some(format!("{target}:{}", srv.port))
	}

	/// Create a `CraftConnection` from an already-established `TcpStream`. This is for server-side
	/// connections accepted from a listener; clients should use [`CraftConnection::connect`] so the
	/// dialed hostname is preserved. Sets the `TcpStream` to use `nodelay`, returning an error if it fails to do so.
	///
	/// Set client_type to [ConnectionRole::CLIENT] if this is a client, or [ConnectionRole::SERVER] if this is a server's connection to a client.
	pub fn from_connection(tcp_stream: TcpStream, client_type: ConnectionRole) -> Result<Self, NetworkError> {
		tcp_stream.set_nodelay(true)?; // disable Nagle's algorithm - according to WIKI specs

		Ok(Self {
			socket_addr: tcp_stream.peer_addr()?,
			hostname: None,
			tcp_stream,
			packet_state: PacketState::HANDSHAKING,
			compression_threshold: None,
			protocol_version: None,
			client_type,
			craft_cipher: None,
			read_buffer: Vec::with_capacity(1024),
			pending_frame: Vec::new(),
		})
	}

	/// Send a minecraft packet to the client. This will block until the packet is sent.
	pub async fn send_packet(&mut self, packet: Packet) -> Result<(), NetworkError> {
		let mut serializer = McSerializer::new();
		packet.mc_serialize(&mut serializer)?;
		let output = &serializer.output;

		trace!("Sending to {self} : {output:?}");

		let mut prefix_deserializer = McDeserializer::new(output);
		VarInt::mc_deserialize(&mut prefix_deserializer)?;
		let prefix_len = prefix_deserializer.index;
		let body = &output[prefix_len..];

		let mut frame = McSerializer::new();
		if let Some(threshold) = self.compression_threshold {
			if body.len() >= threshold as usize {
				let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
				enc.write_all(body)?;
				let compressed = enc.finish()?;

				let mut inner = McSerializer::new();
				VarInt(body.len() as i32).mc_serialize(&mut inner)?;
				inner.output.extend_from_slice(&compressed);

				VarInt(inner.output.len() as i32).mc_serialize(&mut frame)?;
				frame.merge(inner);
			} else {
				let mut inner = McSerializer::new();
				VarInt(0).mc_serialize(&mut inner)?;
				inner.output.extend_from_slice(body);

				VarInt(inner.output.len() as i32).mc_serialize(&mut frame)?;
				frame.merge(inner);
			}
			trace!("Compressed packet for {self} : {} bytes compressed to {} bytes", body.len(), frame.output.len());
		} else {
			frame.output.extend_from_slice(output);
		}

		// Encrypt the fully-framed packet after compression, if encryption is enabled.
		if let Some(cipher) = &mut self.craft_cipher {
			cipher.encrypt(&mut frame.output);
		}

		self.tcp_stream.write_all(&frame.output).await?;

		Ok(())
	}

	/// Given the body of a received packet frame (everything after the outer length VarInt),
	/// produce a length-prefixed buffer (`VarInt(len) + Packet ID + Data`) ready for
	/// `Packet::deserialize_state`.
	///
	/// When compression is enabled the frame body begins with a Data Length VarInt followed by
	/// either the raw Packet ID + Data (Data Length == 0, packet was below the threshold) or the
	/// zlib-compressed Packet ID + Data (Data Length == uncompressed length).
	///
	/// See <https://minecraft.wiki/w/Java_Edition_protocol/Packets#With_compression>
	fn build_deserializer_buffer(&self, frame_body: &[u8]) -> Result<Vec<u8>, NetworkError> {
		let body: Vec<u8> = if self.compression_threshold.is_some() {
			let mut sub = McDeserializer::new(frame_body);
			let data_length = VarInt::mc_deserialize(&mut sub)?;
			let remaining = &frame_body[sub.index..];

			if data_length.0 == 0 {
				// Packet was below the threshold and sent uncompressed
				remaining.to_vec()
			} else {
				if data_length.0 as usize > PACKET_MAX_SIZE {
					return Err(NetworkError::PacketTooLarge);
				}

				let mut decoder = ZlibDecoder::new(remaining);
				let mut decompressed = Vec::with_capacity(data_length.0 as usize);
				decoder.read_to_end(&mut decompressed)?;
				decompressed
			}
		} else {
			frame_body.to_vec()
		};

		let mut serializer = McSerializer::new();
		VarInt(body.len() as i32).mc_serialize(&mut serializer)?;
		serializer.output.extend_from_slice(&body);
		Ok(serializer.output)
	}

	/// Receive a minecraft packet from the client. This will block until a packet is received. This removes data from the TCP buffer
	pub async fn receive_packet(&mut self) -> Result<Packet, NetworkError> {
		// Finish a frame that try_receive_packet started, or its bytes are lost and the stream goes out of sync.
		while !self.pending_frame.is_empty() {
			match self.try_receive_packet() {
				Err(NetworkError::IOError(e)) if e.kind() == std::io::ErrorKind::WouldBlock => self.tcp_stream.readable().await?,
				Err(NetworkError::NoDataReceived) => {
					self.close().await;
					return Err(NetworkError::NoDataReceived);
				}
				result => return result,
			}
		}

		// Read VarInt length prefix using a stack array (no heap allocation)
		let mut varint_buf = [0u8; 3];
		let mut varint_len = 0usize;

		loop {
			let mut b = self.tcp_stream.read_u8().await?;

			if let Some(cipher) = &mut self.craft_cipher {
				let mut one = [b];
				cipher.decrypt(&mut one);
				b = one[0];
			}

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

		if let Some(cipher) = &mut self.craft_cipher {
			cipher.decrypt(&mut self.read_buffer[varint_len..]);
		}

		let buffer = self.build_deserializer_buffer(&self.read_buffer[varint_len..total_len])?;
		let mut deserializer = McDeserializer::new(&buffer);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, self.client_type)?;

		Ok(packet)
	}

	/// Try to receive a packet from the buffer without blocking.
	///
	/// If the full frame is not available yet, this returns `NetworkError::IOError` with kind
	/// `WouldBlock`. The bytes received so far are kept, and the next call continues the same frame.
	/// Returns `NoDataReceived` if the peer closed the connection.
	///
	/// [CraftConnection::receive_packet] also continues a frame that this function started.
	pub fn try_receive_packet(&mut self) -> Result<Packet, NetworkError> {
		// Read the length prefix one byte at a time, so no byte of the next frame is consumed.
		let (var_len, total_len) = loop {
			if let Some(end) = self.pending_frame.iter().position(|b| b & CONTINUE_BIT == 0) {
				let packet_len = VarInt::from_slice(&self.pending_frame[..=end])?.0 as usize;
				if packet_len > PACKET_MAX_SIZE {
					self.pending_frame.clear();
					return Err(NetworkError::PacketTooLarge);
				}
				break (end + 1, end + 1 + packet_len);
			} else if self.pending_frame.len() >= 3 {
				self.pending_frame.clear();
				return Err(SerializingErr::VarTypeTooLong("Packet length VarInt max bytes is 3".to_string()).into());
			}

			self.try_read_pending(1)?;
		};

		while self.pending_frame.len() < total_len {
			self.try_read_pending(total_len - self.pending_frame.len())?;
		}

		let frame = std::mem::take(&mut self.pending_frame);

		trace!("Received from {} : {:?}", self, &frame);

		let deser_buf = self.build_deserializer_buffer(&frame[var_len..])?;
		let mut deserializer = McDeserializer::new(&deser_buf);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, self.client_type)?;

		Ok(packet)
	}

	/// Read up to `n` more bytes of the current frame into `pending_frame` without blocking, and
	/// decrypt them if encryption is enabled.
	fn try_read_pending(&mut self, n: usize) -> Result<(), NetworkError> {
		let start = self.pending_frame.len();
		self.pending_frame.resize(start + n, 0);

		let read = match self.tcp_stream.try_read(&mut self.pending_frame[start..]) {
			Ok(read) => read,
			Err(e) => {
				self.pending_frame.truncate(start);
				return Err(e.into());
			}
		};
		self.pending_frame.truncate(start + read);

		if read == 0 {
			return Err(NetworkError::NoDataReceived);
		}

		if let Some(cipher) = &mut self.craft_cipher {
			cipher.decrypt(&mut self.pending_frame[start..]);
		}

		Ok(())
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
	///
	/// Cannot be used if encryption is enabled.
	pub async fn peek_packet(&mut self) -> Result<Packet, NetworkError> {
		if self.craft_cipher.is_some() {
			return Err(NetworkError::PeekNotSupportedWhileEncrypted);
		}

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

		// peek always reads from the start of the stream, so peek the full frame (length prefix included)
		let length = match self.tcp_stream.peek(&mut buffer).await {
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

		let deser_buf = self.build_deserializer_buffer(&buffer[varint_len..])?;
		let mut deserializer = McDeserializer::new(&deser_buf);
		let packet = Packet::deserialize_state(&mut deserializer, self.packet_state, self.client_type)?;

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

		if length == 0 {
			// connection closed
			self.close().await;
			return Err(NetworkError::NoDataReceived);
		}

		Ok(buffer)
	}

	/// Change the internal Packet State. This is used to categorize what kind of packets are being sent/received.
	/// See [PacketState] for more information.
	///
	/// This does not inform the connected server or client of the change in state.
	pub fn change_state(&mut self, state: PacketState) {
		self.packet_state = state;
	}

	/// Enable compression on the connection. This will compress packets that are larger than the threshold.
	///
	/// Set `threshold` to `None` to disable compression, or to `Some(value)` to enable compression with the given threshold in bytes.
	///
	/// Packets cannot be larger than 2^21 − 1 or 2097151 bytes (the maximum that can be sent in a 3-byte VarInt).
	pub fn enable_compression(&mut self, threshold: Option<u32>) {
		self.compression_threshold = threshold;
	}

	/// Set the shared secret and enable encryption on this connection. All further packets sent and received will be encrypted with this shared secret.
	/// This should be called after the handshake and encryption request/response have been completed.
	///
	/// See [server::server_handler_trait::ServerEncryptionHandler] to auto-handle encryption handshakes for incoming connections.
	pub fn set_shared_secret(&mut self, secret: &[u8]) -> Result<(), NetworkError> {
		let secret = match <[u8; 16]>::try_from(secret) {
			Ok(s) => s,
			Err(_) => {
				return Err(NetworkError::EncryptionError(crate::util::encryption::EncryptionError::InvalidSharedSecretLength(secret.len())));
			}
		};

		self.craft_cipher = Some(CraftCipher::new(&secret)?);

		Ok(())
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

		write!(f, "CraftConnection: {s}")
	}
}
