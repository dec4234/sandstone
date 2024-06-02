use std::io::Cursor;

use base64::Engine;
use base64::engine::general_purpose;
use image::{DynamicImage, ImageFormat};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::packets::packets::packet::StatusResponseBody;
use crate::packets::serialization::serializer_error::SerializingErr;
use crate::packets::serialization::serializer_handler::{DeserializeResult, McDeserialize, McDeserializer, McSerialize, McSerializer};
use crate::protocol_details::protocol_verison::ProtocolVerison;

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

	pub fn set_favicon_image(&mut self, image: DynamicImage) {
		let mut image_data: Vec<u8> = Vec::new();
		image.write_to(&mut Cursor::new(&mut image_data), ImageFormat::Png)
			.unwrap();
		let res_base64 = general_purpose::STANDARD.encode(image_data);
		let s = format!("data:image/png;base64,{}", res_base64);

		self.favicon = Some(s);
	}

	pub fn set_secure_chat(&mut self, secure: bool) {
		self.enforcesSecureChat = secure;
	}

	pub fn set_preview_chat(&mut self, preview: bool) {
		self.previewsChat = preview;
	}

	pub fn set_description(&mut self, description: String) {
		self.description = DescriptionInfo {
			text: description
		};
	}

	pub fn set_player_info(&mut self, max: i32, online: i32, sample: Vec<PlayerSample>) {
		self.players = PlayerInfo {
			max,
			online,
			sample
		};
	}

	/// *version* can really be anything you want, but *protocol_version* must be a valid protocol version number
	pub fn set_protocol_version(&mut self, version: String, protocol_version: i16) {
		self.version = VersionInfo {
			name: version,
			protocol: protocol_version,
		};
	}
}

impl McSerialize for StatusResponseSpec {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> Result<(), SerializingErr> {
		let serialized = serde_json::to_string(self).unwrap();

		serialized.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for StatusResponseSpec {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> DeserializeResult<'a, Self> where Self: Sized {
		let raw = serde_json::from_str(&String::mc_deserialize(deserializer)?).unwrap(); // TODO: test

		Ok(raw)
	}
}

impl From<StatusResponseBody> for StatusResponseSpec {
	fn from(p: StatusResponseBody) -> Self {
		p.response
	}
}

impl From<StatusResponseSpec> for StatusResponseBody {
	fn from(p: StatusResponseSpec) -> Self {
		StatusResponseBody {
			response: p
		}
	}
}

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DescriptionInfo { // TODO: update to chat thing?
	text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerSample {
	name: String,
	id: String,
}

impl PlayerSample {
	pub fn new<S: Into<String>>(name: S, id: Uuid) -> Self {
		Self {
			name: name.into().replace("&", "§"),
			id: id.to_string(),
		}
	}

	pub fn new_random<S: Into<String>>(name: S) -> Self {
		Self {
			name: name.into().replace("&", "§"),
			id: Uuid::new_v4().to_string(), // TODO: no-std support?
		}
	}
}

