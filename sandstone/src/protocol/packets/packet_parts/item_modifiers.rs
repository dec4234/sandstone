use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, VarIntEnum};

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum DyeColor {
	White = 0,
	Orange = 1,
	Magenta = 2,
	LightBlue = 3,
	Yellow = 4,
	Lime = 5,
	Pink = 6,
	Gray = 7,
	LightGray = 8,
	Cyan = 9,
	Purple = 10,
	Blue = 11,
	Brown = 12,
	Green = 13,
	Red = 14,
	Black = 15,
}
#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn dye_color_ids_match_wiki() {
		let ser = |c: DyeColor| {
			let mut s = McSerializer::new();
			c.mc_serialize(&mut s).unwrap();
			s.as_bytes().to_vec()
		};
		assert_eq!(ser(DyeColor::White), vec![0]);
		assert_eq!(ser(DyeColor::LightGray), vec![8]);
		assert_eq!(ser(DyeColor::Black), vec![15]);
		assert_eq!(DyeColor::mc_deserialize(&mut McDeserializer::new(&[14])).unwrap(), DyeColor::Red);
		assert!(DyeColor::mc_deserialize(&mut McDeserializer::new(&[16])).is_err());
	}
}
