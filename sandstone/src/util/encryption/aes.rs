//! AES/CFB8 stream ciphers used to encrypt traffic after the login handshake.
//!
//! The ciphers are updated continuously for the lifetime of the connection: a single
//! [`CraftCipher`] must be reused across every packet, never reset per packet, or the
//! keystream will desynchronize from the peer.
//!
//! <https://minecraft.wiki/w/Java_Edition_protocol/Encryption>

use aes::Aes128;
use aes::cipher::array::Array;
use aes::cipher::{Block, BlockModeDecrypt, BlockModeEncrypt, KeyIvInit};
use rand::{Rng, rng};

use crate::util::encryption::EncryptionError;

/// AES-128 CFB-8 encryptor. The "8" segment size (1 byte) is mandated by the protocol.
pub type Aes128Cfb8Enc = cfb8::Encryptor<Aes128>;
/// AES-128 CFB-8 decryptor.
pub type Aes128Cfb8Dec = cfb8::Decryptor<Aes128>;

/// Length in bytes of the AES shared secret (128-bit).
pub const SHARED_SECRET_LEN: usize = 16;

/// Generates a random 16-byte shared secret, seeded from the OS RNG.
///
/// The client produces this when it receives an Encryption Request, encrypts it with the
/// server's public key, and then uses it as the AES key/IV for the rest of the connection.
pub fn generate_shared_secret() -> [u8; SHARED_SECRET_LEN] {
	let mut secret = [0u8; SHARED_SECRET_LEN];
	rng().fill_bytes(&mut secret);
	secret
}

/// A pair of continuously-updated AES/CFB8 ciphers for a single connection.
///
/// Both the key and IV are set to the shared secret, matching the vanilla protocol. The
/// same instance must be reused for every packet.
#[derive(Debug)]
pub struct CraftCipher {
	encryptor: CraftEncryptor,
	decryptor: CraftDecryptor,
}

/// The encryption half of a [`CraftCipher`], for the write half of a split connection.
#[derive(Debug)]
pub struct CraftEncryptor(Aes128Cfb8Enc);

/// The decryption half of a [`CraftCipher`], for the read half of a split connection.
#[derive(Debug)]
pub struct CraftDecryptor(Aes128Cfb8Dec);

impl CraftEncryptor {
	/// Encrypts `data` in place, advancing the encryption cipher state.
	pub fn encrypt(&mut self, data: &mut [u8]) {
		for byte in data.iter_mut() {
			let mut block: Block<Aes128Cfb8Enc> = Array::from([*byte]);
			self.0.encrypt_block(&mut block);
			*byte = block[0];
		}
	}
}

impl CraftDecryptor {
	/// Decrypts `data` in place, advancing the decryption cipher state.
	pub fn decrypt(&mut self, data: &mut [u8]) {
		for byte in data.iter_mut() {
			let mut block: Block<Aes128Cfb8Dec> = Array::from([*byte]);
			self.0.decrypt_block(&mut block);
			*byte = block[0];
		}
	}
}

impl CraftCipher {
	/// Builds the cipher pair from the shared secret. Fails with
	/// [`EncryptionError::InvalidSharedSecretLength`] if the secret is not
	/// [`SHARED_SECRET_LEN`] bytes long.
	pub fn new(shared_secret: &[u8]) -> Result<Self, EncryptionError> {
		if shared_secret.len() != SHARED_SECRET_LEN {
			return Err(EncryptionError::InvalidSharedSecretLength(shared_secret.len()));
		}

		// Key and IV are both the shared secret.
		let encryptor = Aes128Cfb8Enc::new_from_slices(shared_secret, shared_secret).map_err(|_| EncryptionError::InvalidSharedSecretLength(shared_secret.len()))?;
		let decryptor = Aes128Cfb8Dec::new_from_slices(shared_secret, shared_secret).map_err(|_| EncryptionError::InvalidSharedSecretLength(shared_secret.len()))?;

		Ok(Self {
			encryptor: CraftEncryptor(encryptor),
			decryptor: CraftDecryptor(decryptor),
		})
	}

	/// Encrypts `data` in place, advancing the encryption cipher state.
	pub fn encrypt(&mut self, data: &mut [u8]) {
		self.encryptor.encrypt(data);
	}

	/// Decrypts `data` in place, advancing the decryption cipher state.
	pub fn decrypt(&mut self, data: &mut [u8]) {
		self.decryptor.decrypt(data);
	}

	/// Split into the two halves. Each half keeps its current state, so the keystreams stay in
	/// sync with the peer.
	pub fn split(self) -> (CraftEncryptor, CraftDecryptor) {
		(self.encryptor, self.decryptor)
	}
}
