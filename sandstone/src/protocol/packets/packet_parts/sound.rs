use crate::bitflag;
use crate::protocol::game::effects::sound::SoundEvent;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::internal_types::IDorX;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

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

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct InlineSoundEvent {
	pub name: String,
	pub has_fixed_range: bool,
	#[mc(deserialize_if = has_fixed_range)]
	pub fixed_range: Option<f32>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct JukeboxSong {
	pub sound_event: IDorX<SoundEvent>,
	pub description: TextComponent,
	pub duration: f32,
	pub output: VarInt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdOrJukeboxSong {
	Registry(VarInt),
	Inline(Box<JukeboxSong>),
}

impl McSerialize for IdOrJukeboxSong {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			IdOrJukeboxSong::Registry(id) => {
				VarInt(id.0 + 1).mc_serialize(serializer)?;
			}
			IdOrJukeboxSong::Inline(song) => {
				VarInt(0).mc_serialize(serializer)?;
				song.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for IdOrJukeboxSong {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> where Self: Sized {
		let typ = VarInt::mc_deserialize(deserializer)?.0;
		if typ == 0 {
			Ok(IdOrJukeboxSong::Inline(Box::new(JukeboxSong::mc_deserialize(deserializer)?)))
		} else {
			Ok(IdOrJukeboxSong::Registry(VarInt(typ - 1)))
		}
	}
}

impl McDefault for IdOrJukeboxSong {
	fn mc_default() -> Self {
		IdOrJukeboxSong::Registry(VarInt(0))
	}
}