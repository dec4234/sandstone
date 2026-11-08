//! Authentication against Mojang's session servers.
//!
//! This covers the encryption handshake used during login: generating the server ID
//! hash, the client telling Mojang it has joined a server, the server verifying that
//! join, and fetching Mojang's signing public keys.

use crate::http::{HttpError, MojangServerQueryClient};
use crate::profile::SkinPropertyWrapper;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use uuid::Uuid;

const SESSION_SERVER: &str = "https://sessionserver.mojang.com";
const MINECRAFT_SERVICES: &str = "https://api.minecraftservices.com";

/// Computes the server ID hash used during the login encryption handshake.
///
/// This mirrors Mojang's `generateServerId` algorithm: a SHA-1 digest over the
/// base server ID, the shared AES secret key, and the server's RSA public key, then
/// rendered as a signed hex string (Java's `new BigInteger(digest).toString(16)`),
/// which can be negative and is therefore prefixed with `-` and has leading zeros stripped.
///
/// `base_server_id` is usually an empty string. Its bytes are taken as ISO-8859-1;
/// for the typical empty/ASCII value this is identical to UTF-8.
/// `public_key` and `secret_key` are the DER/encoded key bytes.
pub fn generate_server_id(base_server_id: &str, public_key: &[u8], secret_key: &[u8]) -> String {
	let mut hasher = Sha1::new();
	hasher.update(base_server_id.as_bytes());
	hasher.update(secret_key);
	hasher.update(public_key);
	let mut digest: Vec<u8> = hasher.finalize().to_vec();

	// Java interprets the digest as a signed two's-complement big integer. If the high
	// bit is set the value is negative, so negate the magnitude and emit a leading '-'.
	let negative = digest[0] & 0x80 != 0;
	if negative {
		let mut carry = true;
		for b in digest.iter_mut().rev() {
			*b = !*b;
			if carry {
				let (v, overflow) = b.overflowing_add(1);
				*b = v;
				carry = overflow;
			}
		}
	}

	let mut hex = String::with_capacity(digest.len() * 2);
	for b in &digest {
		hex.push_str(&format!("{:02x}", b));
	}
	let hex = hex.trim_start_matches('0');
	let hex = if hex.is_empty() {
		"0"
	} else {
		hex
	};

	if negative {
		format!("-{}", hex)
	} else {
		hex.to_string()
	}
}

/// Body sent by the client to verify a login session (`/session/minecraft/join`).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct JoinServerRequest {
	/// The Minecraft access token.
	pub accessToken: String,
	/// The player's UUID without dashes.
	pub selectedProfile: String,
	/// The server ID produced by [`generate_server_id`].
	pub serverId: String,
}

/// Verifies a login session on the client side.
///
/// POSTs to `https://sessionserver.mojang.com/session/minecraft/join`. Mojang responds
/// with HTTP 204 on success, so this returns `Ok(())` when authentication passes.
/// This endpoint is rate limited to 6 joins per 30 seconds per account.
pub async fn join_server(access_token: String, selected_profile: Uuid, server_id: String) -> Result<(), HttpError> {
	join_server_at(SESSION_SERVER, access_token, selected_profile, server_id).await
}

pub(crate) async fn join_server_at(base_url: &str, access_token: String, selected_profile: Uuid, server_id: String) -> Result<(), HttpError> {
	let body = JoinServerRequest {
		accessToken: access_token,
		selectedProfile: selected_profile.to_string().replace("-", ""),
		serverId: server_id,
	};

	let body = serde_json::to_string(&body)?;

	MojangServerQueryClient::post(format!("{}/session/minecraft/join", base_url), body).await?;

	Ok(())
}

/// Response returned when a server verifies a login session via `hasJoined`.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HasJoinedResponse {
	/// The player's UUID without dashes.
	pub id: String,
	/// The player's case-sensitive name.
	pub name: String,
	/// Signed player properties (e.g. the `textures` property).
	pub properties: Vec<SkinPropertyWrapper>,
}

/// Verifies a login session on the server side.
///
/// GETs `https://sessionserver.mojang.com/session/minecraft/hasJoined`. The `ip` argument
/// is optional. On success Mojang returns the player's signed profile; if verification
/// fails Mojang returns HTTP 204 with an empty body, which surfaces here as an error.
pub async fn has_joined(username: String, server_id: String, ip: Option<String>) -> Result<HasJoinedResponse, HttpError> {
	has_joined_at(SESSION_SERVER, username, server_id, ip).await
}

pub(crate) async fn has_joined_at(base_url: &str, username: String, server_id: String, ip: Option<String>) -> Result<HasJoinedResponse, HttpError> {
	let mut map: HashMap<&str, &str> = HashMap::new();
	map.insert("username", username.as_str());
	map.insert("serverId", server_id.as_str());
	if let Some(ip) = ip.as_ref() {
		map.insert("ip", ip.as_str());
	}

	let text = MojangServerQueryClient::get_params(format!("{}/session/minecraft/hasJoined", base_url), map).await?;

	parse_has_joined(&text)
}

/// Parses a `hasJoined` response body. An empty body (HTTP 204) means the player is not authenticated.
pub(crate) fn parse_has_joined(text: &str) -> Result<HasJoinedResponse, HttpError> {
	if text.trim().is_empty() {
		return Err(HttpError::NotAuthenticated);
	}

	Ok(serde_json::from_str::<HasJoinedResponse>(text)?)
}

/// A single Base64-encoded DER public key returned by the `publickeys` endpoint.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MojangPublicKey {
	pub publicKey: String,
}

/// Response from the Mojang public keys endpoint.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MojangPublicKeysResponse {
	/// Keys used to verify player properties such as skin and cape textures.
	pub profilePropertyKeys: Vec<MojangPublicKey>,
	/// Keys used to verify player public keys / certificates.
	pub playerCertificateKeys: Vec<MojangPublicKey>,
	/// Keys used to verify authentication tokens (e.g. JWT access tokens).
	pub authenticationKeys: Vec<MojangPublicKey>,
}

/// Fetches Mojang's signing public keys.
///
/// GETs `https://api.minecraftservices.com/publickeys`. These keys are used to verify
/// player properties, player certificates and authentication tokens.
pub async fn get_mojang_public_keys() -> Result<MojangPublicKeysResponse, HttpError> {
	get_mojang_public_keys_at(MINECRAFT_SERVICES).await
}

pub(crate) async fn get_mojang_public_keys_at(base_url: &str) -> Result<MojangPublicKeysResponse, HttpError> {
	MojangServerQueryClient::get_parse(format!("{}/publickeys", base_url), false).await
}

#[cfg(test)]
mod tests {
	use super::*;
	use mockito::Matcher;

	const FIXTURE_PROFILE: &str = include_str!("../tests/fixtures/profile_dec4234.json");

	/// The client join request must use the exact field names Mojang expects and send the
	/// UUID without dashes. With any other shape Mojang rejects the join and the player
	/// cannot log in to an online-mode server.
	#[tokio::test]
	async fn join_server_posts_expected_body() {
		let mut server = mockito::Server::new_async().await;
		let mock = server
			.mock("POST", "/session/minecraft/join")
			.match_header("content-type", "application/json")
			.match_body(Matcher::Json(serde_json::json!({
				"accessToken": "token123",
				"selectedProfile": "ef39c1973c3d4776a22622096378a966",
				"serverId": "-1428ae341e1f2602f07e3ef662ad847db5aaab1e",
			})))
			.with_status(204)
			.create_async()
			.await;

		let uuid = Uuid::parse_str("ef39c197-3c3d-4776-a226-22096378a966").unwrap();
		join_server_at(&server.url(), "token123".into(), uuid, "-1428ae341e1f2602f07e3ef662ad847db5aaab1e".into())
			.await
			.unwrap();

		mock.assert_async().await;
	}

	/// Mojang answers a bad or expired access token with 403. This must be an error so the
	/// client stops the login instead of sending the encryption response.
	#[tokio::test]
	async fn join_server_rejected_is_error() {
		let mut server = mockito::Server::new_async().await;
		server.mock("POST", "/session/minecraft/join").with_status(403).create_async().await;

		let err = join_server_at(&server.url(), "bad".into(), Uuid::nil(), "id".into()).await.unwrap_err();

		assert!(matches!(err, HttpError::StatusCode(_)));
	}

	/// Without an IP, only `username` and `serverId` may be sent. An empty `ip` parameter
	/// would make Mojang compare against a wrong address.
	#[tokio::test]
	async fn has_joined_sends_username_and_server_id_only() {
		let mut server = mockito::Server::new_async().await;
		let mock = server
			.mock("GET", "/session/minecraft/hasJoined")
			.match_query(Matcher::Regex("^(username=dec4234&serverId=abc|serverId=abc&username=dec4234)$".into()))
			.with_body(FIXTURE_PROFILE)
			.create_async()
			.await;

		let resp = has_joined_at(&server.url(), "dec4234".into(), "abc".into(), None).await.unwrap();

		assert_eq!(resp.name, "dec4234");
		assert_eq!(resp.id, "ef39c1973c3d4776a22622096378a966");
		mock.assert_async().await;
	}

	/// When the server gives the player's IP, Mojang must receive it, because it uses it to
	/// block a stolen session that joins from a different address.
	#[tokio::test]
	async fn has_joined_sends_ip_when_given() {
		let mut server = mockito::Server::new_async().await;
		let mock = server
			.mock("GET", "/session/minecraft/hasJoined")
			.match_query(Matcher::AllOf(vec![
				Matcher::UrlEncoded("username".into(), "dec4234".into()),
				Matcher::UrlEncoded("serverId".into(), "abc".into()),
				Matcher::UrlEncoded("ip".into(), "203.0.113.7".into()),
			]))
			.with_body(FIXTURE_PROFILE)
			.create_async()
			.await;

		has_joined_at(&server.url(), "dec4234".into(), "abc".into(), Some("203.0.113.7".into())).await.unwrap();

		mock.assert_async().await;
	}

	/// The full path: a real HTTP 204 from Mojang must become `NotAuthenticated` so the
	/// server disconnects the player.
	#[tokio::test]
	async fn has_joined_no_content_is_not_authenticated() {
		let mut server = mockito::Server::new_async().await;
		server.mock("GET", "/session/minecraft/hasJoined").match_query(Matcher::Any).with_status(204).create_async().await;

		let err = has_joined_at(&server.url(), "dec4234".into(), "abc".into(), None).await.unwrap_err();

		assert!(matches!(err, HttpError::NotAuthenticated));
	}

	/// The server uses these keys to check signed player data, so the request must go to
	/// `/publickeys` and every key group must reach the caller.
	#[tokio::test]
	async fn get_public_keys_requests_and_parses() {
		let mut server = mockito::Server::new_async().await;
		let mock = server
			.mock("GET", "/publickeys")
			.with_body(include_str!("../tests/fixtures/public_keys.json"))
			.create_async()
			.await;

		let resp = get_mojang_public_keys_at(&server.url()).await.unwrap();

		assert!(!resp.profilePropertyKeys.is_empty());
		assert!(!resp.playerCertificateKeys.is_empty());
		assert!(!resp.authenticationKeys.is_empty());
		mock.assert_async().await;
	}

	/// Mojang answers a failed session check with HTTP 204 and no body. This must be an
	/// explicit `NotAuthenticated` error so a server rejects the player instead of
	/// treating a parse failure (or worse, success) as the result.
	#[test]
	fn parse_has_joined_empty_body_is_not_authenticated() {
		assert!(matches!(parse_has_joined(""), Err(HttpError::NotAuthenticated)));
		assert!(matches!(parse_has_joined("  \n"), Err(HttpError::NotAuthenticated)));
	}

	/// A successful check returns the same profile shape as the session profile endpoint.
	/// The server needs the id, name and signed textures property to spawn the player.
	#[test]
	fn parse_has_joined_profile_fixture() {
		let resp = parse_has_joined(include_str!("../tests/fixtures/profile_dec4234.json")).unwrap();

		assert_eq!(resp.id, "ef39c1973c3d4776a22622096378a966");
		assert_eq!(resp.name, "dec4234");
		assert_eq!(resp.properties[0].name, "textures");
		assert!(resp.properties[0].signature.is_some());
	}
}
