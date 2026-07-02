//! Cryptographic primitives for the Minecraft login encryption handshake.
//!
//! Reference: https://minecraft.wiki/w/Java_Edition_protocol/Encryption

pub mod aes;
pub mod rsa;

use thiserror::Error;

/// Any error that can occur while setting up or running the login encryption handshake.
#[derive(Error, Debug)]
pub enum EncryptionError {
	/// The AES shared secret was not [`aes::SHARED_SECRET_LEN`] bytes long.
	#[error("invalid shared secret length: expected {expected} bytes, got {0}", expected = aes::SHARED_SECRET_LEN)]
	InvalidSharedSecretLength(usize),
	/// An RSA operation (key generation, encryption, or decryption) failed.
	#[error(transparent)]
	Rsa(#[from] ::rsa::Error),
	/// Encoding or decoding an X.509 `SubjectPublicKeyInfo` public key failed.
	#[error(transparent)]
	Spki(#[from] ::rsa::pkcs8::spki::Error),
	#[error("verify token mismatch: expected {expected:?}, got {actual:?}")]
	MismatchedVerifyToken {
		/// The expected verify token.
		expected: Vec<u8>,
		/// The actual verify token received from the client.
		actual: Vec<u8>,
	},
}

#[cfg(test)]
mod tests {
	use super::aes::{generate_shared_secret, CraftCipher, SHARED_SECRET_LEN};
	use super::rsa::{decode_public_key, decrypt_with_private_key, encode_public_key, encrypt_with_public_key, generate_key_pair, generate_verify_token, RSA_KEY_BITS, VERIFY_TOKEN_LEN};
	use super::EncryptionError;
	use ::rsa::RsaPublicKey;

	/// The DER public key must survive a round trip, because the client decodes exactly
	/// these bytes from the Encryption Request before it can encrypt anything. If encoding
	/// and decoding disagreed, the whole handshake would silently fail.
	#[test]
	fn public_key_der_round_trip() {
		let private = generate_key_pair().unwrap();
		let public = RsaPublicKey::from(&private);

		let der = encode_public_key(&public).unwrap();
		assert!(!der.is_empty(), "encoded SubjectPublicKeyInfo should not be empty");

		let decoded = decode_public_key(&der).unwrap();
		assert_eq!(decoded, public, "decoding the DER bytes must reproduce the original key");
	}

	/// End-to-end key exchange: the client encrypts the shared secret and verify token
	/// with the public key it decoded from the wire, and the server recovers the exact
	/// same bytes with its private key. This is the property the login handshake relies on.
	#[test]
	fn shared_secret_and_token_exchange() {
		let private = generate_key_pair().unwrap();
		let server_public_der = encode_public_key(&RsaPublicKey::from(&private)).unwrap();

		// Client side: decode the server's key, then encrypt the secret and token.
		let client_view = decode_public_key(&server_public_der).unwrap();
		let shared_secret = generate_shared_secret();
		let verify_token = generate_verify_token();

		let enc_secret = encrypt_with_public_key(&client_view, &shared_secret).unwrap();
		let enc_token = encrypt_with_public_key(&client_view, &verify_token).unwrap();

		// Both ciphertexts are 128 bytes for a 1024-bit key, per the protocol.
		assert_eq!(enc_secret.len(), RSA_KEY_BITS / 8);
		assert_eq!(enc_token.len(), RSA_KEY_BITS / 8);

		// Server side: decrypt and confirm the values match what the client sent.
		let dec_secret = decrypt_with_private_key(&private, &enc_secret).unwrap();
		let dec_token = decrypt_with_private_key(&private, &enc_token).unwrap();

		assert_eq!(dec_secret, shared_secret);
		assert_eq!(dec_token, verify_token);
	}

	#[test]
	fn generated_values_have_expected_lengths() {
		assert_eq!(generate_shared_secret().len(), SHARED_SECRET_LEN);
		assert_eq!(generate_verify_token().len(), VERIFY_TOKEN_LEN);
	}

	/// A wrong-sized secret must be rejected at construction rather than producing a
	/// cipher that silently disagrees with the peer.
	#[test]
	fn cipher_rejects_wrong_secret_length() {
		match CraftCipher::new(&[0u8; 8]) {
			Err(EncryptionError::InvalidSharedSecretLength(8)) => {}
			_ => panic!("expected InvalidSharedSecretLength(8)"),
		}
	}

	/// The client's encryptor and the server's decryptor are configured identically, so
	/// data the client encrypts must decrypt back to the original on the server.
	#[test]
	fn aes_round_trip_between_peers() {
		let secret = generate_shared_secret();
		let mut client = CraftCipher::new(&secret).unwrap();
		let mut server = CraftCipher::new(&secret).unwrap();

		let plaintext = b"the quick brown fox jumps over the lazy dog".to_vec();
		let mut buf = plaintext.clone();

		client.encrypt(&mut buf);
		assert_ne!(buf, plaintext, "ciphertext should differ from plaintext");

		server.decrypt(&mut buf);
		assert_eq!(buf, plaintext, "server must recover what the client encrypted");
	}

	/// The cipher state is continuous across packets — encrypting a stream in one call or
	/// in several smaller calls must produce identical ciphertext. This is *why*
	/// [`CraftCipher`] holds long-lived ciphers instead of recreating them per packet.
	#[test]
	fn aes_state_is_continuous_across_chunks() {
		let secret = generate_shared_secret();
		let data = (0u8..=200).collect::<Vec<u8>>();

		// Encrypt the whole stream at once.
		let mut whole = data.clone();
		CraftCipher::new(&secret).unwrap().encrypt(&mut whole);

		// Encrypt the same stream split into two calls on one continuous cipher.
		let mut chunked = data.clone();
		let mut cipher = CraftCipher::new(&secret).unwrap();
		let (a, b) = chunked.split_at_mut(50);
		cipher.encrypt(a);
		cipher.encrypt(b);

		assert_eq!(whole, chunked, "chunking must not change the keystream");

		// And it still decrypts, even when the receiver chunks differently.
		let mut decryptor = CraftCipher::new(&secret).unwrap();
		let (c, d) = chunked.split_at_mut(120);
		decryptor.decrypt(c);
		decryptor.decrypt(d);
		assert_eq!(chunked, data);
	}
}
