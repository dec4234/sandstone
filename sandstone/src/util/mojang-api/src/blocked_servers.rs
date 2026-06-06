use crate::http::HttpError;
use std::collections::HashSet;

const BLOCKED_SERVERS_URL: &str = "https://sessionserver.mojang.com/blockedservers";

/// Returns the list of blocked server SHA1 hashes published by Mojang.
///
/// Each line is a hex-encoded SHA1 hash (40 chars) of a blocked server address.
pub async fn get_blocked_servers() -> Result<HashSet<String>, HttpError> {
	let response = crate::http::MojangServerQueryClient::get(BLOCKED_SERVERS_URL.to_string()).await?;

	let mut vec = HashSet::new();

	for line in response.lines() {
		let line = line.trim();
		if line.is_empty() {
			continue;
		}
		vec.insert(line.to_string());
	}

	Ok(vec)
}