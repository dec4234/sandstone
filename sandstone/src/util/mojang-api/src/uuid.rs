//! Username -> UUID lookups via the Mojang API.

use crate::http::{HttpError, MojangServerQueryClient};
use serde::{Deserialize, Serialize};

/// Get the UUID of a username
/// This will return an error if it exceeds the rate limit or if no user with the given username exists
pub async fn get_uuid_from_username(name: String) -> Result<UuidRequestResponse, HttpError> {
	let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", name);

	MojangServerQueryClient::get_parse(url, false).await
}

/// Response returned when trying to get a UUID from a username
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UuidRequestResponse {
	pub id: String,
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub legacy: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub demo: Option<bool>,
}

/// Get the UUIDs of multiple usernames at once, in alphabetical order
/// This will return an error if it exceeds the rate limit
pub async fn get_uuids_from_usernames(names: Vec<String>) -> Result<Vec<UuidRequestResponse>, HttpError> {
	let body = serde_json::to_string(&names)?;

	MojangServerQueryClient::post_parse("https://api.minecraftservices.com/minecraft/profile/lookup/bulk/byname", body.as_str(), false).await
}
