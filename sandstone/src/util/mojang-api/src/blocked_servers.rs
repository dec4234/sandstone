use crate::http::HttpError;
use std::collections::HashSet;

const BLOCKED_SERVERS_URL: &str = "https://sessionserver.mojang.com/blockedservers";

/// Returns the list of blocked server SHA1 hashes published by Mojang.
///
/// Each line is a hex-encoded SHA1 hash (40 chars) of a blocked server address.
pub async fn get_blocked_servers() -> Result<HashSet<String>, HttpError> {
	let response = crate::http::MojangServerQueryClient::get(BLOCKED_SERVERS_URL.to_string()).await?;

	Ok(parse_blocked_servers(&response))
}

/// Parses the newline-separated hash list returned by the blocked servers endpoint.
pub(crate) fn parse_blocked_servers(response: &str) -> HashSet<String> {
	let mut vec = HashSet::new();

	for line in response.lines() {
		let line = line.trim();
		if line.is_empty() {
			continue;
		}
		vec.insert(line.to_string());
	}

	vec
}

#[cfg(test)]
mod tests {
	use super::*;

	/// A server checks joins against this set, so blank or padded lines must not become
	/// entries and real hashes must match exactly (no whitespace).
	#[test]
	fn parse_blocked_servers_fixture() {
		let set = parse_blocked_servers(include_str!("../tests/fixtures/blocked_servers.txt"));

		assert_eq!(set.len(), 3);
		assert!(set.contains("dc2c735b3e6aba51ece294d7de21b947379aac4d"));
		assert!(!set.contains(""));
	}
}
