use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::util::mojang::http::ApiClient;

/*
This file defines the Mojang API - used to get information about users, servers and encryption validation
The rate limit is allegedly 600 requests per 10 minutes
Reference = https://wiki.vg/Mojang_API
 */

pub async fn get_uuid_from_username(name: String) -> Result<UuidRequestResponse> {
	let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", name);
	
	Ok(ApiClient::new().enable_debug_mode().await.get_parse(url, false).await?)
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UuidRequestResponse {
	pub id: String,
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub legacy: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub demo: Option<bool>
}