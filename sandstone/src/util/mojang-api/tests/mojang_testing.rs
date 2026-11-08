//! Testing Mojang API endpoints and processing.
//!
//! Offline tests parse saved real responses from `tests/fixtures/` and run with a plain
//! `cargo test`. The `#[ignore]` tests call the live Mojang API, which is rate limited.
//! Run them only when needed, one at a time:
//! `cargo test -p mojang-api -- --ignored --test-threads=1`

/// Tests against saved Mojang responses. These need no network and cannot be rate limited.
/// Each checks that a real response shape still deserializes into our public types.
#[cfg(test)]
mod offline {
	use mojang_api::{MojangPublicKeysResponse, PlayerDetailsResponse, UuidRequestResponse};

	#[test]
	fn uuid_from_username_fixture() {
		let resp: UuidRequestResponse = serde_json::from_str(include_str!("fixtures/uuid_notch.json")).unwrap();

		assert_eq!(resp.id, "069a79f444e94726a5befca90e38aaf5");
		assert_eq!(resp.name, "Notch");
		assert_eq!(resp.legacy, None);
		assert_eq!(resp.demo, None);
	}

	/// Mojang returns the bulk lookup in an unspecified order, so match by name, not index.
	#[test]
	fn bulk_usernames_fixture() {
		let resp: Vec<UuidRequestResponse> = serde_json::from_str(include_str!("fixtures/uuid_bulk.json")).unwrap();

		assert_eq!(resp.len(), 4);
		let id_of = |name: &str| resp.iter().find(|r| r.name == name).map(|r| r.id.as_str());
		assert_eq!(id_of("Notch"), Some("069a79f444e94726a5befca90e38aaf5"));
		assert_eq!(id_of("jeb_"), Some("853c80ef3c3749fdaa49938b674adae6"));
		assert_eq!(id_of("dec4234"), Some("ef39c1973c3d4776a22622096378a966"));
		// Mojang returns the canonical capitalisation, not the requested "dinnerbone".
		assert_eq!(id_of("Dinnerbone"), Some("61699b2ed3274a019f1e0ea8c3f06bc6"));
	}

	/// The textures property is base64 JSON inside JSON. Decoding it must yield the skin
	/// URL, because that is what a server sends to clients to render the player.
	#[test]
	fn player_details_fixture() {
		let resp: PlayerDetailsResponse = serde_json::from_str(include_str!("fixtures/profile_dec4234.json")).unwrap();

		assert_eq!(resp.id, "ef39c1973c3d4776a22622096378a966");
		assert_eq!(resp.name, "dec4234");
		assert_eq!(resp.profileActions.len(), 0);
		assert_eq!(resp.properties.len(), 1);
		assert_eq!(resp.properties[0].name, "textures");

		let skin = resp.properties[0].get_skin_details().unwrap();
		assert_eq!(skin.profileId, resp.id);
		assert_eq!(skin.profileName, resp.name);
		assert!(skin.textures.SKIN.expect("textures should contain a skin").url.starts_with("http"));
	}

	#[test]
	fn public_keys_fixture() {
		let resp: MojangPublicKeysResponse = serde_json::from_str(include_str!("fixtures/public_keys.json")).unwrap();

		assert!(!resp.profilePropertyKeys.is_empty());
		assert!(!resp.playerCertificateKeys.is_empty());
		assert!(!resp.authenticationKeys.is_empty());
	}
}

#[cfg(test)]
mod test {
	use mojang_api::{generate_server_id, get_blocked_servers, get_mojang_public_keys, get_player_details, get_uuid_from_username, get_uuids_from_usernames, has_joined, join_server};
	use rand_chacha::rand_core::SeedableRng;
	use rsa::pkcs8::EncodePublicKey;
	use rsa::{PrivateKey, PublicKey};
	use uuid::Uuid;

	/// Generates static, deterministic fake credentials for tests that need an RSA public key
	/// and an AES secret key (e.g. the encryption handshake / server ID hash). The RNG is
	/// seeded with a fixed value so the same key pair is produced on every run, which keeps
	/// any derived value (such as the server ID) stable and assertable.
	///
	/// Returns `(public_key_der, secret_key)`, mirroring what Mojang feeds to its server ID
	/// algorithm: the RSA public key as X.509 DER bytes and the raw 128-bit AES secret key.
	fn fake_credentials() -> (Vec<u8>, Vec<u8>) {
		let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
		let private = PrivateKey::new(&mut rng, 1024).expect("failed to generate fake RSA key");
		let public = PublicKey::from(&private);
		let public_der = public.to_public_key_der().expect("failed to encode public key").as_bytes().to_vec();

		let secret_key = vec![0x42u8; 16];

		(public_der, secret_key)
	}

	#[ignore]
	#[tokio::test]
	pub async fn test_get_uuid_from_name() {
		let response = get_uuid_from_username("Notch".to_string()).await.unwrap();

		assert_eq!(response.id, "069a79f444e94726a5befca90e38aaf5");
		assert_eq!(response.name, "Notch");
		assert_eq!(response.legacy, None);
		assert_eq!(response.demo, None);

		let response = get_uuid_from_username("jeb_".to_string()).await.unwrap();

		assert_eq!(response.id, "853c80ef3c3749fdaa49938b674adae6");
		assert_eq!(response.name, "jeb_");
		assert_eq!(response.legacy, None);
		assert_eq!(response.demo, None);

		tokio::time::sleep(std::time::Duration::from_secs(1)).await; // too fast

		let response = get_uuid_from_username("dec4234".to_string()).await.unwrap();

		assert_eq!(response.id, "ef39c1973c3d4776a22622096378a966");
		assert_eq!(response.name, "dec4234");
		assert_eq!(response.legacy, None);
		assert_eq!(response.demo, None);

		let response = get_uuid_from_username("dinnerbone".to_string()).await.unwrap();

		assert_eq!(response.id, "61699b2ed3274a019f1e0ea8c3f06bc6");
		assert_eq!(response.name, "Dinnerbone");
		assert_eq!(response.legacy, None);
		assert_eq!(response.demo, None);

		tokio::time::sleep(std::time::Duration::from_secs(1)).await; // too fast

		// people that don't exist -- should return an error that the given username doesn't exist
		get_uuid_from_username("37g43g3i4yid".to_string()).await.expect_err("Expected error");
		get_uuid_from_username("sdewve3veyirv3r".to_string()).await.expect_err("Expected error");
		get_uuid_from_username("22___343n3irt72".to_string()).await.expect_err("Expected error");
		get_uuid_from_username("ABC__23n2al1l".to_string()).await.expect_err("Expected error");
	}

	#[ignore]
	#[tokio::test]
	pub async fn test_bulk_usernames() {
		let resp = get_uuids_from_usernames(vec!["Notch".to_string(), "jeb_".to_string(), "dec4234".to_string(), "dinnerbone".to_string()])
			.await
			.unwrap();

		assert_eq!(resp.len(), 4);
		assert_eq!(resp[0].id, "ef39c1973c3d4776a22622096378a966");
		assert_eq!(resp[0].name, "dec4234");

		assert_eq!(resp[1].id, "61699b2ed3274a019f1e0ea8c3f06bc6");
		assert_eq!(resp[1].name, "Dinnerbone");

		assert_eq!(resp[2].id, "853c80ef3c3749fdaa49938b674adae6");
		assert_eq!(resp[2].name, "jeb_");

		assert_eq!(resp[3].id, "069a79f444e94726a5befca90e38aaf5");
		assert_eq!(resp[3].name, "Notch");
	}

	#[ignore]
	#[tokio::test]
	pub async fn test_get_player_details() {
		let response = get_player_details("ef39c1973c3d4776a22622096378a966".to_string()).await.unwrap();
		assert_eq!(response.id, "ef39c1973c3d4776a22622096378a966");
		assert_eq!(response.name, "dec4234");
		assert_eq!(response.legacy, None);
		assert_eq!(response.profileActions.len(), 0);

		assert_eq!(response.properties.len(), 1);
		assert_eq!(response.properties[0].name, "textures");

		let _texture = response.properties[0].get_skin_details().unwrap();
	}

	#[ignore]
	#[tokio::test]
	pub async fn test_blocked_servers() {
		get_blocked_servers().await.unwrap();
		println!("Blocked servers: {:?}", get_blocked_servers().await.unwrap());
	}

	/// `generate_server_id` mirrors Mojang's `generateServerId`. With empty secret/public
	/// key bytes the digest reduces to SHA-1 of the base string, so we can pin it against
	/// the well-known reference vectors (including the signed/negative case for "jeb_").
	#[tokio::test]
	pub async fn test_generate_server_id_mojang_benchmarks() {
		// Algorithm correctness against Mojang's published reference vectors (empty keys, so
		// the digest reduces to SHA-1 of the base string).
		assert_eq!(generate_server_id("Notch", &[], &[]), "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48");
		assert_eq!(generate_server_id("jeb_", &[], &[]), "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1");
		assert_eq!(generate_server_id("simon", &[], &[]), "88e16a1019277b15d58faf0541e11910eb756f6");
	}

	#[tokio::test]
	pub async fn test_generate_server_id_real() {
		let (public_key, secret_key) = fake_credentials();

		assert_eq!(generate_server_id("dec4234", &public_key, &secret_key), "7909d97f30a8e26fc5b25eeaff576c2a930276e5");
		assert_eq!(generate_server_id("jeb_", &public_key, &secret_key), "-1428ae341e1f2602f07e3ef662ad847db5aaab1e");
		assert_eq!(generate_server_id("simon", &public_key, &secret_key), "7cbd3bcb8a2040a77a2479b11b48abb50881fdb6");

		// The static fake credentials seed a fixed key pair, so the server ID is stable.
		let (public_key, secret_key) = fake_credentials();
		assert_eq!(generate_server_id("", &public_key, &secret_key), "b2c362c0d5cf8e4c1dae428d8f842913571ec8");
	}

	#[ignore]
	#[tokio::test]
	pub async fn test_join_server() {
		let (public_key, secret_key) = fake_credentials();
		let server_id = generate_server_id("", &public_key, &secret_key);

		join_server("".to_string(), Uuid::new_v4(), server_id).await.expect_err("expected rejection for a bogus access token");
	}

	/// A real success requires the client above to have just joined with the same server ID,
	/// so it cannot run unattended. Without a matching join Mojang returns HTTP 204 with an
	/// empty body, which fails to deserialize, so we assert the error path here.
	#[ignore]
	#[tokio::test]
	pub async fn test_has_joined() {
		let (public_key, secret_key) = fake_credentials();
		let server_id = generate_server_id("", &public_key, &secret_key);

		has_joined("dec4234".to_string(), server_id, None).await.expect_err("expected failure without a matching join");
	}

	#[ignore]
	#[tokio::test]
	pub async fn test_get_mojang_public_keys() {
		let response = get_mojang_public_keys().await.unwrap();

		assert!(!response.profilePropertyKeys.is_empty());
		assert!(!response.playerCertificateKeys.is_empty());
		assert!(!response.authenticationKeys.is_empty());
	}
}
