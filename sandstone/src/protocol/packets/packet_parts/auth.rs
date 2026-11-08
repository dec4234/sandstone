use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::testing::McDefault;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

/// # Public Key (Packet Part)
/// Type used to communicate a public key on network.
#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct PublicKeyNetwork {
	pub expires_at: i64,
	pub public_key: PrefixedArray<u8>,
	pub key_signature: PrefixedArray<u8>
}
#[cfg(test)]
mod test {
	use super::*;

	fn ser<T: McSerialize>(v: &T) -> Vec<u8> {
		let mut s = McSerializer::new();
		v.mc_serialize(&mut s).unwrap();
		s.as_bytes().to_vec()
	}

	fn round_trip<T: McSerialize + McDeserialize + PartialEq + std::fmt::Debug>(v: T) {
		let bytes = ser(&v);
		let mut d = McDeserializer::new(&bytes);
		assert_eq!(T::mc_deserialize(&mut d).unwrap(), v);
		assert!(d.is_at_end(), "all bytes must be used");
	}

	#[test]
	fn public_key_layout() {
		// Expires At is a Long (8 bytes), then the key and the signature as length-prefixed bytes.
		let k = PublicKeyNetwork { expires_at: 1, public_key: PrefixedArray::new(vec![0xAA, 0xBB]), key_signature: PrefixedArray::new(vec![0xCC]) };
		assert_eq!(ser(&k), vec![0, 0, 0, 0, 0, 0, 0, 1, 2, 0xAA, 0xBB, 1, 0xCC]);
		round_trip(k);
	}
}
