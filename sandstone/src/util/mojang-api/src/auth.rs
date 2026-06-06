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
    let hex = if hex.is_empty() { "0" } else { hex };

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
    let body = JoinServerRequest {
        accessToken: access_token,
        selectedProfile: selected_profile.to_string().replace("-", ""),
        serverId: server_id,
    };

    let body = serde_json::to_string(&body)?;

    MojangServerQueryClient::post("https://sessionserver.mojang.com/session/minecraft/join", body.as_str()).await?;

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
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("username", username.as_str());
    map.insert("serverId", server_id.as_str());
    if let Some(ip) = ip.as_ref() {
        map.insert("ip", ip.as_str());
    }

    MojangServerQueryClient::get_parse_params("https://sessionserver.mojang.com/session/minecraft/hasJoined".to_string(), false, map).await
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
    MojangServerQueryClient::get_parse("https://api.minecraftservices.com/publickeys".to_string(), false).await
}