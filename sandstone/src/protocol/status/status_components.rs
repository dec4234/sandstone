use std::io::Cursor;

use base64::Engine;
use base64::engine::general_purpose;
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::protocol::packets::StatusResponsePacket;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol_types::protocol_verison::ProtocolVerison;

/// A prepared response to a status request from a client. This provides useful functions for building
/// the complicated nested structure of the status response.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct StatusResponseSpec {
	version: VersionInfo,
	players: PlayerInfo,
	description: DescriptionInfo,
	#[serde(skip_serializing_if = "Option::is_none")]
	favicon: Option<String>,
	enforcesSecureChat: bool,
	previewsChat: bool,
}

impl StatusResponseSpec {
	/// Create a new status response with the given protocol version and description. The description
	/// will have its color codes translated from the symbol '&' to the symbol '§'.
	pub fn new<T: Into<String>>(protocol_version: ProtocolVerison, description: T) -> Self {
		Self {
			version: VersionInfo {
				name: protocol_version.get_fancy_name(),
				protocol: protocol_version.get_version_number(),
			},
			players: PlayerInfo {
				max: 0,
				online: 0,
				sample: Vec::new(),
			},
			description: DescriptionInfo {
				text: description.into().replace("&", "§")
			},
			favicon: None,
			enforcesSecureChat: false,
			previewsChat: false,
		}
	}

	/// Set the image returned to the user as the server logo.
	/// This must be a 64x64 PNG image.
	pub fn set_favicon_image(&mut self, image: DynamicImage) {
		let mut image_data: Vec<u8> = Vec::new();
		image.write_to(&mut Cursor::new(&mut image_data), ImageFormat::Png)
			.unwrap();
		let res_base64 = general_purpose::STANDARD.encode(image_data);
		let s = format!("data:image/png;base64,{}", res_base64);

		self.favicon = Some(s);
	}

	/// Unknown purpose. Might be related to post 1.18 chat security.
	pub fn set_secure_chat(&mut self, secure: bool) {
		self.enforcesSecureChat = secure;
	}

	/// Unknown purpose. Might be related to post 1.18 chat security.
	pub fn set_preview_chat(&mut self, preview: bool) {
		self.previewsChat = preview;
	}

	/// Set the description/MOTD of the server, which is displayed in the server list.
	/// The description will have its color codes translated from the symbol '&' to the symbol '§'.
	pub fn set_description(&mut self, description: String) {
		self.description = DescriptionInfo {
			text: description
		};
	}

	/// Set the player list preview response, seen when the user hovers over the player count.
	pub fn set_player_info(&mut self, max: i32, online: i32, sample: Vec<PlayerSample>) {
		self.players = PlayerInfo {
			max,
			online,
			sample
		};
	}

	/// `version` can really be anything you want, but `protocol_version` must be a valid protocol version number
	pub fn set_protocol_version(&mut self, version: String, protocol_version: i16) {
		self.version = VersionInfo {
			name: version,
			protocol: protocol_version,
		};
	}
}

impl McSerialize for StatusResponseSpec {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let serialized = serde_json::to_string(self).unwrap();

		serialized.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for StatusResponseSpec {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let raw = serde_json::from_str(&String::mc_deserialize(deserializer)?).unwrap(); // TODO: test

		Ok(raw)
	}
}

impl From<StatusResponsePacket> for StatusResponseSpec {
	fn from(p: StatusResponsePacket) -> Self {
		p.response
	}
}

impl From<StatusResponseSpec> for StatusResponsePacket {
	fn from(p: StatusResponseSpec) -> Self {
		StatusResponsePacket {
			response: p
		}
	}
}

/// Represents the version information for the server. The `name` of the version can be anything you want.
/// The `protocol` must be a valid protocol version number, and must match the protocol version of the
/// connecting client.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionInfo {
	name: String,
	protocol: i16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerInfo {
	max: i32,
	online: i32,
	sample: Vec<PlayerSample>,
}

impl PlayerInfo {
	pub fn add_player(&mut self, player: PlayerSample) {
		self.sample.push(player);
	}

	pub fn set_player_sample(&mut self, players: Vec<PlayerSample>) {
		self.sample = players;
	}
}

/// Represents the description/MOTD of the server, which is displayed in the server list.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DescriptionInfo { // TODO: update to chat thing?
	text: String,
}

/// Represents a single entry in the player list sample response, seen when the user hovers over the player count.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerSample {
	name: String,
	id: String,
}

impl PlayerSample {
	/// Create a new player sample with the given name and UUID.
	/// The name will have its color codes translated from the symbol '&' to the symbol '§'.
	pub fn new<S: Into<String>>(name: S, id: Uuid) -> Self {
		Self {
			name: name.into().replace("&", "§"),
			id: id.to_string(),
		}
	}

	/// Create a new player sample with the given name and a random UUID.
	/// The name will have its color codes translated from the symbol '&' to the symbol '§'.
	pub fn new_random<S: Into<String>>(name: S) -> Self {
		Self {
			name: name.into().replace("&", "§"),
			id: Uuid::new_v4().to_string(), // TODO: no-std support?
		}
	}
}

