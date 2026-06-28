//! Status protocol components.

use crate::protocol::testing::McDefault;
use std::io::Cursor;

use crate::protocol::packets::StatusResponsePacket;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::protocol_verison::ProtocolVerison;
use base64::engine::general_purpose;
use base64::Engine;
use image::{DynamicImage, ImageFormat};
use sandstone_derive::McDefault;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A prepared response to a status request from a client. This provides useful functions for building
/// the complicated nested structure of the status response.
#[derive(McDefault, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[allow(non_snake_case)]
pub struct StatusResponseSpec {
	pub version: VersionInfo,
	// `players` is optional; omitted by servers that hide their player list.
	#[serde(default)]
	pub players: PlayerInfo,
	pub description: TextComponent,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub favicon: Option<String>,
	// Optional/non-standard trailing fields: vanilla third-party servers (e.g. Hypixel) omit
	// `enforcesSecureChat`, and `previewsChat` is not part of the modern protocol at all.
	#[serde(default)]
	pub enforcesSecureChat: bool,
	#[serde(default)]
	pub previewsChat: bool,
}

impl StatusResponseSpec {
	/// Create a new status response with the given protocol version and description. The description
	/// will have its color codes translated from the symbol '&' to the symbol '§'.
	pub fn new<T: Into<String>>(protocol_version: ProtocolVerison, description: T) -> Self {
		Self {
			version: VersionInfo {
				name: protocol_version.get_fancy_name(),
				protocol: protocol_version,
			},
			players: PlayerInfo {
				max: 0,
				online: 0,
				sample: Vec::new(),
			},
			// color code translation needed here because client uses section symbol for color
			description: TextComponent::new(description.into().replace("&", "§")),
			favicon: None,
			enforcesSecureChat: false,
			previewsChat: false,
		}
	}

	/// Set the image returned to the user as the server logo.
	/// This must be a 64x64 PNG image.
	pub fn set_favicon_image(&mut self, image: DynamicImage) {
		let mut image_data: Vec<u8> = Vec::new();
		image.write_to(&mut Cursor::new(&mut image_data), ImageFormat::Png).unwrap();
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
		self.description = TextComponent::new(description);
	}

	/// Set the player list preview response, seen when the user hovers over the player count.
	pub fn set_player_info(&mut self, max: i32, online: i32, sample: Vec<PlayerSample>) {
		self.players = PlayerInfo {
			max,
			online,
			sample,
		};
	}

	/// `version` can really be anything you want, but `protocol_version` must be a valid protocol version number
	pub fn set_protocol_version(&mut self, version: String, protocol_version: ProtocolVerison) {
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
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
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
			response: p,
		}
	}
}

/// # Version Info (Packet Part)
/// Represents the version information for the server. The `name` of the version can be anything you want.
/// The `protocol` must be a valid protocol version number, and must match the protocol version of the
/// connecting client.
#[derive(McDefault, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionInfo {
	// On 1.20+ servers `name` may be omitted; the Notchian client treats it as "Old" if so.
	#[serde(default = "version_name_default")]
	pub name: String,
	pub protocol: ProtocolVerison,
}

/// Function for a default version name if the name for the version is omitted by the server.
fn version_name_default() -> String {
	"Old".to_string()
}

/// # Player Info (Packet Part)
/// Player info which includes the current number of online users, max slots for the server, and a sample of online users.
///
/// Many servers will customize this information so it shouldn't be taken literally.
#[derive(McDefault, Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerInfo {
	pub max: i32,
	pub online: i32,
	pub sample: Vec<PlayerSample>,
}

impl PlayerInfo {
	pub fn add_player(&mut self, player: PlayerSample) {
		self.sample.push(player);
	}

	pub fn set_player_sample(&mut self, players: Vec<PlayerSample>) {
		self.sample = players;
	}
}

/// Represents a single entry in the player list sample response, seen when the user hovers over the player count.
#[derive(McDefault, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerSample {
	pub name: String,
	pub id: String,
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
			id: Uuid::new_v4().to_string(),
		}
	}
}

#[cfg(test)]
mod test {
	use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer};
	use crate::protocol::status::status_components::{PlayerSample, StatusResponseSpec};
	use crate::protocol_types::datatypes::chat::ComponentType;
	use crate::protocol_types::protocol_verison::ProtocolVerison;
	use uuid::Uuid;

	#[test]
	fn test_color_code_translation() {
		// `new` must translate the '&' color code symbol into the section symbol '§' used by the client.
		let spec = StatusResponseSpec::new(ProtocolVerison::latest(), "&a&lTest");
		assert_eq!(spec.description.content, ComponentType::Text { text: "§a§lTest".to_string() });
	}

	#[test]
	fn test_new_uses_protocol_version() {
		// `new` must derive version info from the supplied protocol version so the client sees a match.
		let version = ProtocolVerison::latest();
		let spec = StatusResponseSpec::new(version, "Test");
		assert_eq!(spec.version.name, version.get_fancy_name());
		assert_eq!(spec.version.protocol, version);
	}

	#[test]
	fn test_set_protocol_version() {
		let mut spec = StatusResponseSpec::new(ProtocolVerison::latest(), "Test");
		spec.set_protocol_version("Custom 1.0".to_string(), ProtocolVerison::V1_8);
		assert_eq!(spec.version.name, "Custom 1.0");
		assert_eq!(spec.version.protocol, ProtocolVerison::V1_8);
	}

	#[test]
	fn test_set_player_info() {
		// Player info drives the count/hover preview shown in the client server list.
		let mut spec = StatusResponseSpec::new(ProtocolVerison::latest(), "Test");
		let sample = vec![PlayerSample::new_random("Steve")];
		spec.set_player_info(20, 1, sample.clone());
		assert_eq!(spec.players.max, 20);
		assert_eq!(spec.players.online, 1);
		assert_eq!(spec.players.sample, sample);
	}

	#[test]
	fn test_player_sample_color_code_translation() {
		// Player sample names are displayed by the client and must use the section color symbol.
		let sample = PlayerSample::new("&cAdmin", Uuid::nil());
		assert_eq!(sample.name, "§cAdmin");
		assert_eq!(sample.id, Uuid::nil().to_string());
	}

	#[test]
	fn test_set_description_no_translation() {
		// `set_description` intentionally does not translate color codes, unlike `new`.
		let mut spec = StatusResponseSpec::new(ProtocolVerison::latest(), "Test");
		spec.set_description("&aRaw".to_string());
		assert_eq!(spec.description.content, ComponentType::Text { text: "&aRaw".to_string() });
	}

	#[test]
	fn test_chat_flags() {
		let mut spec = StatusResponseSpec::new(ProtocolVerison::latest(), "Test");
		assert!(!spec.enforcesSecureChat);
		assert!(!spec.previewsChat);
		spec.set_secure_chat(true);
		spec.set_preview_chat(true);
		assert!(spec.enforcesSecureChat);
		assert!(spec.previewsChat);
	}

	#[test]
	fn test_serialize_deserialize_round_trip() {
		// The spec travels over the wire as a length-prefixed JSON string; it must survive a round-trip.
		let mut spec = StatusResponseSpec::new(ProtocolVerison::latest(), "&aHello");
		spec.set_player_info(100, 5, vec![PlayerSample::new("Notch", Uuid::nil())]);
		spec.set_secure_chat(true);

		let mut serializer = McSerializer::new();
		spec.mc_serialize(&mut serializer).unwrap();

		let mut deserializer = McDeserializer::new(&serializer.output);
		let decoded = StatusResponseSpec::mc_deserialize(&mut deserializer).unwrap();

		assert_eq!(spec, decoded);
	}
}
