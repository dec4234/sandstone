use crate::bitflag;
use crate::protocol::game::effects::sound::SoundEvent;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
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

/// <https://minecraft.wiki/w/Java_Edition_protocol/Slot_data#Jukebox_Song>
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
	fn stop_sound_flags_match_present_fields() {
		// Wiki (Stop Sound): 0x01 means Source is present, 0x02 means Sound is present. A field is
		// only written when its flag is set, else the client reads the wrong bytes.
		assert_eq!(ser(&StopSoundDetails { source: None, sound: None }), vec![0x00]);
		assert_eq!(ser(&StopSoundDetails { source: Some(SoundSource::Block), sound: None }), vec![0x01, 4]);
		assert_eq!(ser(&StopSoundDetails { source: None, sound: Some("a".into()) }), vec![0x02, 1, b'a']);
		assert_eq!(ser(&StopSoundDetails { source: Some(SoundSource::Voice), sound: Some("a".into()) }), vec![0x03, 9, 1, b'a']);
	}

	#[test]
	fn stop_sound_round_trips_each_combination() {
		round_trip(StopSoundDetails { source: None, sound: None });
		round_trip(StopSoundDetails { source: Some(SoundSource::Master), sound: None });
		round_trip(StopSoundDetails { source: None, sound: Some("minecraft:ambient.cave".into()) });
		round_trip(StopSoundDetails { source: Some(SoundSource::Ambient), sound: Some("minecraft:ambient.cave".into()) });
	}

	#[test]
	fn inline_sound_event_range_only_when_flag_set() {
		let without = InlineSoundEvent { name: "a".into(), has_fixed_range: false, fixed_range: None };
		assert_eq!(ser(&without), vec![1, b'a', 0]);
		round_trip(without);

		let with = InlineSoundEvent { name: "a".into(), has_fixed_range: true, fixed_range: Some(16.0) };
		let mut expected = vec![1, b'a', 1];
		expected.extend_from_slice(&16.0f32.to_be_bytes());
		assert_eq!(ser(&with), expected);
		round_trip(with);
	}

	#[test]
	fn jukebox_song_registry_id_is_offset_by_one() {
		// Wiki (ID or X): 0 means an inline value follows, else the registry ID is the value - 1.
		assert_eq!(ser(&IdOrJukeboxSong::Registry(VarInt(0))), vec![1]);
		assert_eq!(ser(&IdOrJukeboxSong::Registry(VarInt(5))), vec![6]);
		round_trip(IdOrJukeboxSong::Registry(VarInt(5)));
	}

	#[test]
	fn jukebox_song_inline_starts_with_zero() {
		let song = IdOrJukeboxSong::Inline(Box::new(JukeboxSong {
			sound_event: IDorX::Registry(VarInt(3)),
			description: TextComponent::new("Song"),
			duration: 120.0,
			output: VarInt(7),
		}));
		assert_eq!(ser(&song)[0], 0);
		round_trip(song);
	}
}
