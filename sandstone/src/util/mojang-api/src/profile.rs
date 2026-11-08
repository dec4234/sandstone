//! Player profile details and skin data via the Mojang API.

use crate::http::{HttpError, MojangServerQueryClient};
use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};

const SESSION_SERVER: &str = "https://sessionserver.mojang.com";

/// Get details about a given UUID such as the name of the user, a list of moderation actions against their account
/// and most importantly, their skin base64 encoded
pub async fn get_player_details(uuid: String) -> Result<PlayerDetailsResponse, HttpError> {
	get_player_details_at(SESSION_SERVER, uuid).await
}

pub(crate) async fn get_player_details_at(base_url: &str, uuid: String) -> Result<PlayerDetailsResponse, HttpError> {
	let url = format!("{}/session/minecraft/profile/{}?unsigned=false", base_url, uuid);

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

#[cfg(test)]
mod tests {
	use super::*;
	use mockito::Matcher;

	/// `unsigned=false` tells Mojang to include the textures signature. Without it, clients
	/// that need signed textures do not show the player's skin.
	#[tokio::test]
	async fn player_details_requests_signed_profile() {
		let mut server = mockito::Server::new_async().await;
		let mock = server
			.mock("GET", "/session/minecraft/profile/ef39c1973c3d4776a22622096378a966")
			.match_query(Matcher::UrlEncoded("unsigned".into(), "false".into()))
			.with_body(include_str!("../tests/fixtures/profile_dec4234.json"))
			.create_async()
			.await;

		let resp = get_player_details_at(&server.url(), "ef39c1973c3d4776a22622096378a966".into()).await.unwrap();

		assert_eq!(resp.name, "dec4234");
		assert!(resp.properties[0].signature.is_some());
		mock.assert_async().await;
	}

	/// A textures value that is not valid base64 must be a `Base64Error`, not a panic.
	#[test]
	fn skin_details_invalid_base64_is_error() {
		let prop = SkinPropertyWrapper { name: "textures".into(), value: "not base64!!".into(), signature: None };

		assert!(matches!(prop.get_skin_details(), Err(HttpError::Base64Error(_))));
	}

	/// Valid base64 that does not hold the textures JSON must be a `SerdeError`.
	#[test]
	fn skin_details_wrong_json_is_error() {
		let prop = SkinPropertyWrapper { name: "textures".into(), value: general_purpose::STANDARD.encode("{}"), signature: None };

		assert!(matches!(prop.get_skin_details(), Err(HttpError::SerdeError(_))));
	}
}
