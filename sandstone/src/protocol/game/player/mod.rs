use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, VarIntEnum};

pub mod player_action;
pub mod interface;
pub mod inventory;

#[derive(McDefault, VarIntEnum, Debug, Clone, PartialEq)]
pub enum ClientStatusAction {
	PerformRespawn = 0,
	RequestStats = 1,
}

/// Data kept after a respawn.
///
/// [Doc link](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Respawn)
#[derive(McDefault, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RespawnKeptData {
	pub keep_attributes: bool,
	pub keep_metadata: bool,
}

impl McSerialize for RespawnKeptData {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let mut flags = 0u8;

		flags |= self.keep_attributes as u8;
		flags |= (self.keep_metadata as u8) << 1;

		flags.mc_serialize(serializer)?;

		Ok(())
	}
}

impl McDeserialize for RespawnKeptData {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self>
	where
		Self: Sized,
	{
		let flags = u8::mc_deserialize(deserializer)?;

		Ok(Self {
			keep_attributes: (flags & 1) != 0,
			keep_metadata: (flags & 2) != 0,
		})
	}
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
	fn respawn_kept_data_bits_match_wiki() {
		// Wiki (Respawn): 0x01 keep attributes, 0x02 keep metadata.
		assert_eq!(ser(&RespawnKeptData { keep_attributes: false, keep_metadata: false }), vec![0x00]);
		assert_eq!(ser(&RespawnKeptData { keep_attributes: true, keep_metadata: false }), vec![0x01]);
		assert_eq!(ser(&RespawnKeptData { keep_attributes: false, keep_metadata: true }), vec![0x02]);
		assert_eq!(ser(&RespawnKeptData { keep_attributes: true, keep_metadata: true }), vec![0x03]);
	}

	#[test]
	fn respawn_kept_data_round_trips() {
		for a in [false, true] {
			for m in [false, true] {
				round_trip(RespawnKeptData { keep_attributes: a, keep_metadata: m });
			}
		}
	}

	#[test]
	fn client_status_ids_match_wiki() {
		assert_eq!(ser(&ClientStatusAction::PerformRespawn), vec![0]);
		assert_eq!(ser(&ClientStatusAction::RequestStats), vec![1]);
		assert!(ClientStatusAction::mc_deserialize(&mut McDeserializer::new(&[2])).is_err());
	}
}
