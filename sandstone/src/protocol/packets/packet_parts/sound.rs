use crate::bitflag;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, VarIntEnum};

bitflag!(SoundControlFlags: u8 {
	source, sound
});

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum SoundSource {
	Master = 0,
	Music = 1,
	Record = 2,
	Weather = 3,
	Block = 4,
	Hostile = 5,
	Neutral = 6,
	Player = 7,
	Ambient = 8,
	Voice = 9,
}

#[derive(McDefault, Debug, Clone, PartialEq)]
pub struct StopSoundDetails {
	source: Option<SoundSource>,
	sound: Option<String>
}

impl McSerialize for StopSoundDetails {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		let flags = SoundControlFlags::new(self.source.is_some(), self.sound.is_some());
		flags.mc_serialize(serializer)?;

		if let Some(source) = &self.source {
			source.mc_serialize(serializer)?;
		}

		if let Some(sound) = &self.sound {
			sound.mc_serialize(serializer)?;
		}

		Ok(())
	}
}

impl McDeserialize for StopSoundDetails {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let flags = SoundControlFlags::mc_deserialize(deserializer)?;

		let source = if flags.source() {
			Some(SoundSource::mc_deserialize(deserializer)?)
		} else {
			None
		};

		let sound = if flags.sound() {
			Some(String::mc_deserialize(deserializer)?)
		} else {
			None
		};

		Ok(Self {
			source,
			sound,
		})
	}
}

