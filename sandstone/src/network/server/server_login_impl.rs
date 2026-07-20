use crate::network::network_error::NetworkError;
use crate::network::server::server_handler_impl::DefaultServerEncryptionHandler;
use crate::network::server::server_handler_trait::{ServerEncryptionHandler, ServerLoginHandler};
use crate::network::CraftConnection;
use crate::protocol::packets::packet_definer::PacketState;
use crate::protocol::packets::{ClientboundKnownPacksPacket, ClientboundPluginMessagePacket, LoginSuccessPacket, Packet, SetCompressionPacket};
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::serialization::{McSerialize, McSerializer};
use crate::protocol_types::datatypes::var_types::VarInt;
use log::debug;
use smart_default::SmartDefault;
use uuid::Uuid;

#[derive(Debug, Clone, SmartDefault)]
pub struct DefaultServerLoginHandler {
	pub should_authenticate_with_mojang: bool,
	pub compression_threshold: Option<i32>,
	#[default("sandstone".to_string())]
	pub server_brand: String
}

impl ServerLoginHandler for DefaultServerLoginHandler {
	async fn handle_login(&self, connection: &mut CraftConnection) -> Result<(), NetworkError> {
		debug!("Beginning login sequence for {connection}");

		let login_start = connection.receive_packet().await?;
		let username = match login_start {
			Packet::LoginStart(packet) => {
				debug!("Received login start packet from {connection}");
				debug!("Login start packet: {packet:?}");

				packet.username
			}
			_ => {
				panic!("Expected login start packet, got {login_start:?}");
			}
		};

		DefaultServerEncryptionHandler::handle_encryption(connection, self.should_authenticate_with_mojang, username.clone(), None).await?;

		if let Some(compression_threshold) = self.compression_threshold {
			let set_compression = Packet::SetCompression(SetCompressionPacket::new(VarInt(compression_threshold)));
			connection.send_packet(set_compression).await?;
			connection.enable_compression(Some(compression_threshold as u32));
			debug!("Sent set compression to {connection} and enabled compression");
		}

		let login_success = Packet::LoginSuccess(LoginSuccessPacket::new(Uuid::new_v4(), username, PrefixedArray::new(vec![])));
		connection.send_packet(login_success).await?;

		let login_ack = connection.receive_packet().await?;
		match login_ack {
			Packet::LoginAcknowledged(..) => {
				debug!("Received login acknowledged packet from {connection}");
				debug!("Login acknowledged packet: {login_ack:?}");
				connection.change_state(PacketState::CONFIGURATION);
			}
			_ => {
				return Err(NetworkError::ExpectedDifferentPacket(format!("Expected login acknowledged packet, got {login_ack:?}")));
			}
		}

		// todo: test brand
		let mut serializer = McSerializer::new();
		self.server_brand.mc_serialize(&mut serializer)?;

		let server_brand = Packet::ClientboundPluginMessage(ClientboundPluginMessagePacket {
			channel: "minecraft:brand".to_string(),
			data: serializer.output,
		});
		connection.send_packet(server_brand).await?;
		debug!("Sent server brand to {connection}");

		let packs = Packet::ClientboundKnownPacks(ClientboundKnownPacksPacket::new(PrefixedArray::new(vec![])));
		connection.send_packet(packs).await?;

		debug!("Sent clientbound known packs to {connection}");

		loop {
			let packs = connection.receive_packet().await?;
			match packs {
				Packet::ServerboundPluginMessage(..) => {
					// optional: before known packs
					debug!("Received plugin message from {connection}: {packs:?}");
				}
				Packet::ClientInformation(..) => {
					// optional: before known packs
					debug!("Received client information from {connection}: {packs:?}");
				}
				Packet::ServerboundKnownPacks(..) => {
					debug!("Received serverbound known packs from {connection}: {packs:?}");
					break;
				}
				_ => {
					return Err(NetworkError::ExpectedDifferentPacket(format!("Expected serverbound known packs packet, got {packs:?}")));
				}
			}
		}

		todo!()
	}
}