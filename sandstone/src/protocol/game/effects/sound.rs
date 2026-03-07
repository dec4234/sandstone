use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::{McDeserialize, McDeserializer, McSerialize, McSerializer, SerializingResult};
use crate::protocol::testing::McDefault;
use sandstone_derive::{McDefault, McDeserialize, McSerialize};

/// [Doc link](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Sound_Event)
#[derive(McDefault, McSerialize, McDeserialize, Debug, PartialOrd, PartialEq, Clone)]
pub struct SoundEvent {
	pub name: String,
	pub has_fixed_range: bool,
	#[mc(deserialize_if = has_fixed_range)]
	pub fixed_range: Option<f32>
}

#[derive(McDefault, Debug, PartialOrd, PartialEq, Clone)]
pub enum SoundCategory {
	Master,
	Music,
	Records,
	Weather,
	Blocks,
	Hostile,
	Neutral,
	Player,
	Ambient,
	Voice
}

impl McSerialize for SoundCategory {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let value = match self {
			SoundCategory::Master => 0,
			SoundCategory::Music => 1,
			SoundCategory::Records => 2,
			SoundCategory::Weather => 3,
			SoundCategory::Blocks => 4,
			SoundCategory::Hostile => 5,
			SoundCategory::Neutral => 6,
			SoundCategory::Player => 7,
			SoundCategory::Ambient => 8,
			SoundCategory::Voice => 9
		};
		value.mc_serialize(serializer)
	}
}

impl McDeserialize for SoundCategory {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let id = u8::mc_deserialize(deserializer)?;
		match id {
			0 => Ok(SoundCategory::Master),
			1 => Ok(SoundCategory::Music),
			2 => Ok(SoundCategory::Records),
			3 => Ok(SoundCategory::Weather),
			4 => Ok(SoundCategory::Blocks),
			5 => Ok(SoundCategory::Hostile),
			6 => Ok(SoundCategory::Neutral),
			7 => Ok(SoundCategory::Player),
			8 => Ok(SoundCategory::Ambient),
			9 => Ok(SoundCategory::Voice),
			_ => Err(SerializingErr::InvalidEnumValue(id as i8))
		}
	}
}