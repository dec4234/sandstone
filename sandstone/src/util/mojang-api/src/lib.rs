//! This file defines the Mojang API - used to get information about users, servers and encryption validation
//! The rate limit is allegedly 600 requests per 10 minutes
//! Reference = https://minecraft.wiki/w/Mojang_API

#![allow(unused)]
#![allow(non_snake_case)]

mod mojang_testing;
mod http;

use base64::Engine;
use base64::engine::general_purpose;
use serde::{Deserialize, Serialize};
use crate::http::{MojangServerQueryClient, HttpError};

/// Get the UUID of a username
/// This will return an error if it exceeds the rate limit or if no user with the given username exists
pub async fn get_uuid_from_username(name: String) -> Result<UuidRequestResponse, HttpError> {
    let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", name);

    Ok(MojangServerQueryClient::get_parse(url, false).await?)
}

/// Response returned when trying to get a UUID from a username
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UuidRequestResponse {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo: Option<bool>
}

/// Get the UUIDs of multiple usernames at once, in alphabetical order
/// This will return an error if it exceeds the rate limit
pub async fn get_uuids_from_usernames(names: Vec<String>) -> Result<Vec<UuidRequestResponse>, HttpError> {
    let body = serde_json::to_string(&names)?;

    let responses = MojangServerQueryClient::post_parse("https://api.minecraftservices.com/minecraft/profile/lookup/bulk/byname", body.as_str(), false).await?;

    Ok(responses)
}

/// Get details about a given UUID such as the name of the user, a list of moderation actions against their account
/// and most importantly, their skin base64 encoded
pub async fn get_player_details(uuid: String) -> Result<PlayerDetailsResponse, HttpError> {
    let url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{}?unsigned=false", uuid);

    Ok(MojangServerQueryClient::get_parse(url, false).await?)
}

///
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
    pub signature: Option<String>
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
    pub metadata: Option<SkinMetadata>
}

/// Raw Skin data as a String
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkinMetadata {
    pub model: String
}
