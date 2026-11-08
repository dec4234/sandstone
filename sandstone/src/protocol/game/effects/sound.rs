use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

/// [Doc link](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Sound_Event)
#[derive(McDefault, McSerialize, McDeserialize, Debug, PartialOrd, PartialEq, Clone)]
pub struct SoundEvent {
	pub name: String,
	pub has_fixed_range: bool,
	#[mc(deserialize_if = has_fixed_range)]
	pub fixed_range: Option<f32>,
}

/// [Doc link](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Sound_Effect)
#[derive(VarIntEnum, McDefault, Debug, PartialOrd, PartialEq, Clone)]
#[repr(i32)]
pub enum SoundCategory {
	Master = 0,
	Music = 1,
	Records = 2,
	Weather = 3,
	Blocks = 4,
	Hostile = 5,
	Neutral = 6,
	Player = 7,
	Ambient = 8,
	Voice = 9,
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
	fn sound_event_range_only_when_flag_set() {
		// Wiki (Sound Event): Fixed Range is only present if Has Fixed Range is true.
		let without = SoundEvent { name: "a".into(), has_fixed_range: false, fixed_range: None };
		assert_eq!(ser(&without), vec![1, b'a', 0]);
		round_trip(without);

		let with = SoundEvent { name: "a".into(), has_fixed_range: true, fixed_range: Some(32.0) };
		let mut expected = vec![1, b'a', 1];
		expected.extend_from_slice(&32.0f32.to_be_bytes());
		assert_eq!(ser(&with), expected);
		round_trip(with);
	}

	#[test]
	fn sound_category_is_one_varint() {
		// Wiki (Sound Effect / Entity Sound Effect): Sound Category is a VarInt Enum. Values 0..=9
		// fit in one VarInt byte. The deserializer reads one byte, so the serializer must write one.
		let all = [
			SoundCategory::Master,
			SoundCategory::Music,
			SoundCategory::Records,
			SoundCategory::Weather,
			SoundCategory::Blocks,
			SoundCategory::Hostile,
			SoundCategory::Neutral,
			SoundCategory::Player,
			SoundCategory::Ambient,
			SoundCategory::Voice,
		];
		for (id, c) in all.into_iter().enumerate() {
			assert_eq!(ser(&c), vec![id as u8], "{c:?}");
			round_trip(c);
		}
	}

	#[test]
	fn unknown_sound_category_is_rejected() {
		assert!(SoundCategory::mc_deserialize(&mut McDeserializer::new(&[10])).is_err());
	}
}
