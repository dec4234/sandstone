//! Username -> UUID lookups via the Mojang API.

use crate::http::{HttpError, MojangServerQueryClient};
use serde::{Deserialize, Serialize};

const MOJANG_API: &str = "https://api.mojang.com";
const MINECRAFT_SERVICES: &str = "https://api.minecraftservices.com";

/// Get the UUID of a username
/// This will return an error if it exceeds the rate limit or if no user with the given username exists
pub async fn get_uuid_from_username(name: String) -> Result<UuidRequestResponse, HttpError> {
	get_uuid_from_username_at(MOJANG_API, name).await
}

pub(crate) async fn get_uuid_from_username_at(base_url: &str, name: String) -> Result<UuidRequestResponse, HttpError> {
	let url = format!("{}/users/profiles/minecraft/{}", base_url, name);

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
	get_uuids_from_usernames_at(MINECRAFT_SERVICES, names).await
}

pub(crate) async fn get_uuids_from_usernames_at(base_url: &str, names: Vec<String>) -> Result<Vec<UuidRequestResponse>, HttpError> {
	let body = serde_json::to_string(&names)?;

	MojangServerQueryClient::post_parse(format!("{}/minecraft/profile/lookup/bulk/byname", base_url), body, false).await
}

#[cfg(test)]
mod tests {
	use super::*;
	use mockito::Matcher;

	/// The username goes in the path. A wrong path returns another player's UUID or none.
	#[tokio::test]
	async fn uuid_from_username_requests_profile_path() {
		let mut server = mockito::Server::new_async().await;
		let mock = server
			.mock("GET", "/users/profiles/minecraft/Notch")
			.with_body(include_str!("../tests/fixtures/uuid_notch.json"))
			.create_async()
			.await;

		let resp = get_uuid_from_username_at(&server.url(), "Notch".into()).await.unwrap();

		assert_eq!(resp.id, "069a79f444e94726a5befca90e38aaf5");
		assert_eq!(resp.name, "Notch");
		mock.assert_async().await;
	}

	/// Mojang answers an unknown username with 404 and an error JSON body. That body must not
	/// be taken as a profile: the caller must get an error.
	#[tokio::test]
	async fn uuid_from_unknown_username_is_error() {
		let mut server = mockito::Server::new_async().await;
		server
			.mock("GET", "/users/profiles/minecraft/37g43g3i4yid")
			.with_status(404)
			.with_body(r#"{"path":"/users/profiles/minecraft/37g43g3i4yid","errorMessage":"Couldn't find any profile with name 37g43g3i4yid"}"#)
			.create_async()
			.await;

		let err = get_uuid_from_username_at(&server.url(), "37g43g3i4yid".into()).await.unwrap_err();

		assert!(matches!(err, HttpError::StatusCode(_)));
	}

	/// The bulk endpoint takes a JSON array of names as a JSON POST. Each returned profile
	/// must reach the caller.
	#[tokio::test]
	async fn bulk_usernames_posts_json_array() {
		let mut server = mockito::Server::new_async().await;
		let mock = server
			.mock("POST", "/minecraft/profile/lookup/bulk/byname")
			.match_header("content-type", "application/json")
			.match_body(Matcher::Json(serde_json::json!(["Notch", "jeb_", "dec4234", "dinnerbone"])))
			.with_body(include_str!("../tests/fixtures/uuid_bulk.json"))
			.create_async()
			.await;

		let names = vec!["Notch".to_string(), "jeb_".to_string(), "dec4234".to_string(), "dinnerbone".to_string()];
		let resp = get_uuids_from_usernames_at(&server.url(), names).await.unwrap();

		assert_eq!(resp.len(), 4);
		assert!(resp.iter().any(|r| r.name == "Dinnerbone" && r.id == "61699b2ed3274a019f1e0ea8c3f06bc6"));
		mock.assert_async().await;
	}
}
