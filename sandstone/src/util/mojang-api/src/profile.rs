//! Player profile details and skin data via the Mojang API.

use crate::http::{HttpError, MojangServerQueryClient};
use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};

/// Get details about a given UUID such as the name of the user, a list of moderation actions against their account
/// and most importantly, their skin base64 encoded
pub async fn get_player_details(uuid: String) -> Result<PlayerDetailsResponse, HttpError> {
	let url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{}?unsigned=false", uuid);

	MojangServerQueryClient::get_parse(url, false).await
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct PlayerDetailsResponse {
	pub id: String,
	pub name: String,
	pub properties: Vec<SkinPropertyWrapper>,
	pub profileActions: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub legacy: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkinPropertyWrapper {
	pub name: String,
	pub value: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub signature: Option<String>,
}

impl SkinPropertyWrapper {
	pub fn get_skin_details(&self) -> Result<SkinProperty, HttpError> {
		let decoded = general_purpose::STANDARD.decode(&self.value)?;
		let decoded = String::from_utf8(decoded)?;

		Ok(serde_json::from_str(&decoded)?)
	}
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkinProperty {
	pub timestamp: u64,
	pub profileId: String,
	pub profileName: String,
	pub signatureRequired: bool,
	pub textures: SkinTexture,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkinTexture {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub SKIN: Option<URLBlock>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub CAPE: Option<URLBlock>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct URLBlock {
	pub url: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub metadata: Option<SkinMetadata>,
}

/// Raw Skin data as a String
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkinMetadata {
	pub model: String,
}
