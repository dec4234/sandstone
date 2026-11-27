//! The read half and the write half of a [CraftConnection], from [CraftConnection::into_split].
//! Each half can go to its own task.

#[cfg(doc)]
use crate::network::CraftConnection;
use crate::network::network_error::NetworkError;
use crate::network::{ConnectionRole, encode_frame, try_receive_frame};
use crate::protocol::packets::Packet;
use crate::protocol::packets::packet_definer::PacketState;
#[cfg(feature = "encryption")]
use crate::util::encryption::aes::{CraftDecryptor, CraftEncryptor};
use log::trace;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

/// The read half of a [CraftConnection]. It receives packets.
#[derive(Debug)]
pub struct CraftReader {
	pub(super) read_half: OwnedReadHalf,
	pub socket_addr: SocketAddr,
	pub packet_state: PacketState,
	pub compression_threshold: Option<u32>,
	pub client_type: ConnectionRole,
	#[cfg(feature = "encryption")]
	pub(super) decryptor: Option<CraftDecryptor>,
	/// Bytes of a frame that have been received but not yet decoded.
	pub(super) pending_frame: Vec<u8>,
}

impl CraftReader {
	/// Receive a minecraft packet. This waits until a full packet is received.
	///
	/// This is cancel safe: if the future is dropped before it completes, the bytes received so far
	/// are kept, and the next call continues the same frame. Returns `NoDataReceived` if the peer
	/// closed the connection.
	pub async fn receive_packet(&mut self) -> Result<Packet, NetworkError> {
		loop {
			match self.try_receive_packet() {
				// `readable` is cancel safe, and the bytes received so far are in `pending_frame`.
				Err(NetworkError::IOError(e)) if e.kind() == std::io::ErrorKind::WouldBlock => self.read_half.readable().await?,
				result => return result,
			}
		}
	}

	/// Try to receive a packet without blocking. See [CraftConnection::try_receive_packet].
	pub fn try_receive_packet(&mut self) -> Result<Packet, NetworkError> {
		let stream = &self.read_half;
		#[cfg(feature = "encryption")]
		let decryptor = &mut self.decryptor;
		try_receive_frame(
			&mut self.pending_frame,
			|buf| stream.try_read(buf),
			|_bytes| {
				#[cfg(feature = "encryption")]
				if let Some(decryptor) = decryptor.as_mut() {
					decryptor.decrypt(_bytes);
				}
			},
			self.compression_threshold,
			self.packet_state,
			self.client_type,
		)
	}

	/// Change the packet state that received packets are decoded with. See [CraftConnection::change_state].
	pub fn change_state(&mut self, state: PacketState) {
		self.packet_state = state;
	}

	/// Set the compression threshold of received packets. See [CraftConnection::enable_compression].
	pub fn enable_compression(&mut self, threshold: Option<u32>) {
		self.compression_threshold = threshold;
	}
}

/// The write half of a [CraftConnection]. It sends packets.
#[derive(Debug)]
pub struct CraftWriter {
	pub(super) write_half: OwnedWriteHalf,
	pub socket_addr: SocketAddr,
	pub compression_threshold: Option<u32>,
	#[cfg(feature = "encryption")]
	pub(super) encryptor: Option<CraftEncryptor>,
}

impl CraftWriter {
	/// Send a minecraft packet. This waits until the packet is sent.
	/// Takes a `Packet` or a packet body (such as `PingResponsePacket`), which is converted with `Into`.
	pub async fn send_packet(&mut self, packet: impl Into<Packet>) -> Result<(), NetworkError> {
		#[cfg_attr(not(feature = "encryption"), allow(unused_mut))]
		let mut frame = encode_frame(packet.into(), self.compression_threshold)?;

		trace!("Sending to {} : {frame:?}", self.socket_addr);

		#[cfg(feature = "encryption")]
		if let Some(encryptor) = &mut self.encryptor {
			encryptor.encrypt(&mut frame);
		}

		self.write_half.write_all(&frame).await?;
		Ok(())
	}

	/// Set the compression threshold of sent packets. See [CraftConnection::enable_compression].
	pub fn enable_compression(&mut self, threshold: Option<u32>) {
		self.compression_threshold = threshold;
	}

	/// Shut down the write side of the connection. The peer then sees the end of the stream.
	pub async fn close(&mut self) -> bool {
		self.write_half.shutdown().await.is_ok()
	}
}
