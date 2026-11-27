//! This module defines the network protocol for the server and client.
//!
//! This includes data types, serializers, packet implementations and client & server handlers.
//!
//! See the documentation for the [client] and [server] modules for more information on how to use the network API.

use crate::network::network_error::NetworkError;
use crate::protocol::packets::Packet;
use crate::protocol::packets::packet_definer::{PacketDirection, PacketState};
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, StateBasedDeserializer};
use crate::protocol_types::datatypes::var_types::VarInt;
use crate::protocol_types::protocol_verison::ProtocolVerison;
#[cfg(feature = "encryption")]
use crate::util::encryption::aes::CraftCipher;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
#[cfg(feature = "dns")]
use hickory_resolver::Resolver;
#[cfg(feature = "dns")]
use hickory_resolver::proto::rr::RecordRef;
#[cfg(feature = "dns")]
use hickory_resolver::proto::rr::rdata::SRV;
use log::{debug, error, trace};
use std::fmt::Display;
use std::io::{Read, Write};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[cfg(feature = "client")]
pub mod client;
pub mod network_error;
#[cfg(feature = "server")]
pub mod server;
mod split;

pub use split::{CraftReader, CraftWriter};

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
	#[cfg(feature = "encryption")]
	craft_cipher: Option<CraftCipher>,
	/// Bytes of a frame that have been received but not yet decoded. The buffer keeps its capacity
	/// between frames.
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
	#[cfg(feature = "dns")]
	async fn resolve_srv(host: &str) -> Option<String> {
		let resolver = Resolver::builder_tokio().ok()?.build().ok()?;
		let lookup = resolver.srv_lookup(format!("_minecraft._tcp.{host}.")).await.ok()?;
		let srv = lookup.answers().iter().find_map(|r| RecordRef::<SRV>::try_from(r).ok())?;
		let srv = srv.data();
		let target = srv.target.to_utf8();
		let target = target.trim_end_matches('.');
		Some(format!("{target}:{}", srv.port))
	}

	/// Without the `dns` feature there is no SRV lookup, so the caller always dials `host:25565`.
	#[cfg(not(feature = "dns"))]
	async fn resolve_srv(_host: &str) -> Option<String> {
		None
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
			#[cfg(feature = "encryption")]
			craft_cipher: None,
			pending_frame: Vec::new(),
		})
	}

	/// Send a minecraft packet to the client. This will block until the packet is sent.
	/// Takes a `Packet` or a packet body (such as `PingResponsePacket`), which is converted with `Into`.
	pub async fn send_packet(&mut self, packet: impl Into<Packet>) -> Result<(), NetworkError> {
		#[cfg_attr(not(feature = "encryption"), allow(unused_mut))]
		let mut frame = encode_frame(packet.into(), self.compression_threshold)?;

		trace!("Sending to {self} : {frame:?}");

		// Encrypt the fully-framed packet after compression, if encryption is enabled.
		#[cfg(feature = "encryption")]
		if let Some(cipher) = &mut self.craft_cipher {
			cipher.encrypt(&mut frame);
		}

		self.tcp_stream.write_all(&frame).await?;

		Ok(())
	}

	/// Receive a minecraft packet from the client. This waits until a full packet is received.
	///
	/// This is cancel safe: if the future is dropped before it completes (for example in a
	/// `tokio::select!` branch that does not win), the bytes received so far are kept, and the next
	/// call to this function or to [CraftConnection::try_receive_packet] continues the same frame.
	pub async fn receive_packet(&mut self) -> Result<Packet, NetworkError> {
		loop {
			match self.try_receive_packet() {
				// `readable` is cancel safe, and the bytes received so far are in `pending_frame`.
				Err(NetworkError::IOError(e)) if e.kind() == std::io::ErrorKind::WouldBlock => self.tcp_stream.readable().await?,
				Err(NetworkError::NoDataReceived) => {
					self.close().await;
					return Err(NetworkError::NoDataReceived);
				}
				Err(NetworkError::IOError(e)) if e.to_string().contains("An established connection was aborted by the software in your host machine") => {
					debug!("OS Error detected in packet receive, closing the connection: {e}");
					self.close().await;
					return Err(NetworkError::ConnectionAbortedLocally);
				}
				result => return result,
			}
		}
	}

	/// Try to receive a packet from the buffer without blocking.
	///
	/// If the full frame is not available yet, this returns `NetworkError::IOError` with kind
	/// `WouldBlock`. The bytes received so far are kept, and the next call continues the same frame.
	/// Returns `NoDataReceived` if the peer closed the connection.
	///
	/// [CraftConnection::receive_packet] also continues a frame that this function started.
	pub fn try_receive_packet(&mut self) -> Result<Packet, NetworkError> {
		let stream = &self.tcp_stream;
		#[cfg(feature = "encryption")]
		let cipher = &mut self.craft_cipher;
		try_receive_frame(
			&mut self.pending_frame,
			|buf| stream.try_read(buf),
			|_bytes| {
				#[cfg(feature = "encryption")]
				if let Some(cipher) = cipher.as_mut() {
					cipher.decrypt(_bytes);
				}
			},
			self.compression_threshold,
			self.packet_state,
			self.client_type,
		)
	}

	/// Split the connection into a read half and a write half, for example to read and write in
	/// separate tasks. The halves keep the state, the compression and the encryption of the
	/// connection, and a frame that was partly received.
	///
	/// After the split, each half has its own copy of the settings. When the state or the
	/// compression changes (for example at the start of the configuration state), change it on both
	/// halves.
	pub fn into_split(self) -> (CraftReader, CraftWriter) {
		let (read_half, write_half) = self.tcp_stream.into_split();
		#[cfg(feature = "encryption")]
		let (encryptor, decryptor) = match self.craft_cipher {
			Some(cipher) => {
				let (encryptor, decryptor) = cipher.split();
				(Some(encryptor), Some(decryptor))
			}
			None => (None, None),
		};

		let reader = CraftReader {
			read_half,
			socket_addr: self.socket_addr,
			packet_state: self.packet_state,
			compression_threshold: self.compression_threshold,
			client_type: self.client_type,
			#[cfg(feature = "encryption")]
			decryptor,
			pending_frame: self.pending_frame,
		};
		let writer = CraftWriter {
			write_half,
			socket_addr: self.socket_addr,
			compression_threshold: self.compression_threshold,
			#[cfg(feature = "encryption")]
			encryptor,
		};
		(reader, writer)
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
		trace!("Received direct from {} : {:?}", self, buffer);
		let mut deserializer = McDeserializer::new(&buffer);
		let packet = T::mc_deserialize(&mut deserializer)?;
		Ok(packet)
	}

	/// Peek the next packet in the queue without removing it. This will block until a packet is received.
	///
	/// Cannot be used if encryption is enabled.
	pub async fn peek_packet(&mut self) -> Result<Packet, NetworkError> {
		#[cfg(feature = "encryption")]
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

		trace!("Peeked from {} : {:?}", self, buffer);

		if length == 0 {
			self.close().await;
			return Err(NetworkError::NoDataReceived);
		} else if length == PACKET_MAX_SIZE {
			return Err(NetworkError::PacketTooLarge);
		}

		let deser_buf = build_deserializer_buffer(self.compression_threshold, &buffer[varint_len..])?;
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

		trace!("Peeked from {} : {:?}", self, buffer);

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
	#[cfg(feature = "encryption")]
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

/// Serialize `packet` into one frame: the length prefix, and the compression header if
/// `compression_threshold` is set. The frame is not encrypted.
///
/// See <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Packet_format>
fn encode_frame(packet: Packet, compression_threshold: Option<u32>) -> Result<Vec<u8>, NetworkError> {
	let mut serializer = McSerializer::new();
	packet.mc_serialize(&mut serializer)?;
	let output = serializer.output;

	let Some(threshold) = compression_threshold else {
		return Ok(output);
	};

	let mut prefix_deserializer = McDeserializer::new(&output);
	VarInt::mc_deserialize(&mut prefix_deserializer)?;
	let body = &output[prefix_deserializer.index..];

	let mut inner = McSerializer::new();
	if body.len() >= threshold as usize {
		let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
		enc.write_all(body)?;
		VarInt(body.len() as i32).mc_serialize(&mut inner)?;
		inner.output.extend_from_slice(&enc.finish()?);
	} else {
		VarInt(0).mc_serialize(&mut inner)?;
		inner.output.extend_from_slice(body);
	}

	let mut frame = McSerializer::new();
	VarInt(inner.output.len() as i32).mc_serialize(&mut frame)?;
	frame.merge(inner);
	trace!("Compressed packet: {} bytes compressed to {} bytes", body.len(), frame.output.len());
	Ok(frame.output)
}

/// Continue the frame in `pending` without blocking, and decode it when it is complete.
///
/// `try_read` reads from the socket without blocking. `decrypt` decrypts the new bytes in place.
/// When the frame is not complete, this returns `NetworkError::IOError` with kind `WouldBlock`, and
/// `pending` keeps the bytes received so far. Returns `NoDataReceived` if the peer closed the
/// connection.
fn try_receive_frame(
	pending: &mut Vec<u8>,
	mut try_read: impl FnMut(&mut [u8]) -> std::io::Result<usize>,
	mut decrypt: impl FnMut(&mut [u8]),
	compression_threshold: Option<u32>,
	state: PacketState,
	client_type: ConnectionRole,
) -> Result<Packet, NetworkError> {
	// Read up to `n` more bytes of the frame into `pending`.
	let mut read_more = |pending: &mut Vec<u8>, n: usize| -> Result<(), NetworkError> {
		let start = pending.len();
		pending.resize(start + n, 0);
		let read = match try_read(&mut pending[start..]) {
			Ok(read) => read,
			Err(e) => {
				pending.truncate(start);
				return Err(e.into());
			}
		};
		pending.truncate(start + read);
		if read == 0 {
			return Err(NetworkError::NoDataReceived);
		}
		decrypt(&mut pending[start..]);
		Ok(())
	};

	// Read the length prefix one byte at a time, so no byte of the next frame is consumed.
	let (var_len, total_len) = loop {
		if let Some(end) = pending.iter().position(|b| b & CONTINUE_BIT == 0) {
			let packet_len = VarInt::from_slice(&pending[..=end])?.0 as usize;
			if packet_len > PACKET_MAX_SIZE {
				pending.clear();
				return Err(NetworkError::PacketTooLarge);
			}
			break (end + 1, end + 1 + packet_len);
		} else if pending.len() >= 3 {
			pending.clear();
			return Err(SerializingErr::VarTypeTooLong("Packet length VarInt max bytes is 3".to_string()).into());
		}

		read_more(pending, 1)?;
	};

	while pending.len() < total_len {
		read_more(pending, total_len - pending.len())?;
	}

	trace!("Received frame : {pending:?}");

	// The frame is used up, also when it does not decode. Clear it but keep the capacity.
	let packet = build_deserializer_buffer(compression_threshold, &pending[var_len..]).and_then(|buffer| {
		let mut deserializer = McDeserializer::new(&buffer);
		Ok(Packet::deserialize_state(&mut deserializer, state, client_type)?)
	});
	pending.clear();
	packet
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
fn build_deserializer_buffer(compression_threshold: Option<u32>, frame_body: &[u8]) -> Result<Vec<u8>, NetworkError> {
	let body: Vec<u8> = if compression_threshold.is_some() {
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
