use crate::bitflag;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::nbt::NbtCompound;
use crate::protocol_types::datatypes::var_types::VarInt;
use sandstone_derive::{McDefault, McDeserialize, McSerialize, VarIntEnum};

bitflag!(FriendlyFlags: u8 {
	allow_friendly_fire, can_see_invisble_teammates
});

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum ObjectiveNumberFormat {
	Blank = 0,
	Styled(NbtCompound) = 1,
	Fixed(TextComponent) = 2,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum ObjectiveType {
	Integer = 0,
	Hearts = 1,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum NameTagVisibility {
	Always = 0,
	Never = 1,
	HideForOtherTeams = 2,
	HideForOwnTeams = 3,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum CollisionRule {
	Always = 0,
	Never = 1,
	PushOtherTeams = 2,
	PushOwnTeam = 3,
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum TeamColor {
	Black = 0,
	DarkBlue = 1,
	DarkGreen = 2,
	DarkAqua = 3,
	DarkRed = 4,
	DarkPurple = 5,
	Gold = 6,
	Gray = 7,
	DarkGray = 8,
	Blue = 9,
	Green = 10,
	Aqua = 11,
	Red = 12,
	LightPurple = 13,
	Yellow = 14,
	White = 15,
	Obfuscated = 16,
	Bold = 17,
	Strikethrough = 18,
	Underlined = 19,
	Italic = 20,
	Reset = 21,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct CreateTeam {
	pub team_display_name: TextComponent,
	pub friendly_flags: FriendlyFlags,
	pub name_tag_visibility: NameTagVisibility,
	pub collision_rule: CollisionRule,
	pub team_color: TeamColor,
	pub team_prefix: TextComponent,
	pub team_suffix: TextComponent,
	pub entities: PrefixedArray<String>,
}

#[derive(McDefault, McSerialize, McDeserialize, Debug, Clone, PartialEq)]
pub struct UpdateTeamInfo {
	pub team_display_name: TextComponent,
	pub friendly_flags: FriendlyFlags,
	pub name_tag_visibility: NameTagVisibility,
	pub collision_rule: CollisionRule,
	pub team_color: TeamColor,
	pub team_prefix: TextComponent,
	pub team_suffix: TextComponent,
}

/// The `team_details` body of the Update Teams packet. The leading Method byte (see wiki) selects
/// the variant and determines the layout of the rest of the packet. The discriminant is encoded as
/// a single `Byte`, not a VarInt, so this implements `McSerialize`/`McDeserialize` by hand rather
/// than using the enum derive (which would write a VarInt id).
#[derive(McDefault, Debug, Clone, PartialEq)]
#[repr(i8)]
pub enum UpdateTeamOptions {
	CreateTeam(CreateTeam) = 0,
	RemoveTeam = 1,
	UpdateTeamInfo(UpdateTeamInfo) = 2,
	AddEntitiesToTeam(PrefixedArray<String>) = 3,
	RemoveEntitiesFromTeam(PrefixedArray<String>) = 4,
}

impl McSerialize for UpdateTeamOptions {
	fn mc_serialize(&self, serializer: &mut McSerializer) -> SerializingResult<()> {
		match self {
			UpdateTeamOptions::CreateTeam(team) => {
				0i8.mc_serialize(serializer)?;
				team.mc_serialize(serializer)?;
			}
			UpdateTeamOptions::RemoveTeam => {
				1i8.mc_serialize(serializer)?;
			}
			UpdateTeamOptions::UpdateTeamInfo(info) => {
				2i8.mc_serialize(serializer)?;
				info.mc_serialize(serializer)?;
			}
			UpdateTeamOptions::AddEntitiesToTeam(entities) => {
				3i8.mc_serialize(serializer)?;
				entities.mc_serialize(serializer)?;
			}
			UpdateTeamOptions::RemoveEntitiesFromTeam(entities) => {
				4i8.mc_serialize(serializer)?;
				entities.mc_serialize(serializer)?;
			}
		}
		Ok(())
	}
}

impl McDeserialize for UpdateTeamOptions {
	fn mc_deserialize<'a>(deserializer: &'a mut McDeserializer) -> SerializingResult<'a, Self> {
		let method = i8::mc_deserialize(deserializer)?;
		match method {
			0 => Ok(UpdateTeamOptions::CreateTeam(CreateTeam::mc_deserialize(deserializer)?)),
			1 => Ok(UpdateTeamOptions::RemoveTeam),
			2 => Ok(UpdateTeamOptions::UpdateTeamInfo(UpdateTeamInfo::mc_deserialize(deserializer)?)),
			3 => Ok(UpdateTeamOptions::AddEntitiesToTeam(PrefixedArray::mc_deserialize(deserializer)?)),
			4 => Ok(UpdateTeamOptions::RemoveEntitiesFromTeam(PrefixedArray::mc_deserialize(deserializer)?)),
			_ => Err(SerializingErr::OutOfBounds(format!("Invalid UpdateTeamOptions method: {}", method))),
		}
	}
}

#[derive(VarIntEnum, McDefault, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum UpdateScoreFormat {
	Blank = 0,
	Styled(NbtCompound) = 1,
	Fixed(TextComponent) = 2,
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

	fn names(n: &[&str]) -> PrefixedArray<String> {
		PrefixedArray::new(n.iter().map(|s| s.to_string()).collect())
	}

	#[test]
	fn team_method_is_one_byte() {
		// Wiki (Update Teams): Method is a Byte, not a VarInt. These bodies have no other fields
		// before the data, so the first byte must be the method.
		assert_eq!(ser(&UpdateTeamOptions::RemoveTeam), vec![1]);
		assert_eq!(ser(&UpdateTeamOptions::AddEntitiesToTeam(names(&["a"]))), vec![3, 1, 1, b'a']);
		assert_eq!(ser(&UpdateTeamOptions::RemoveEntitiesFromTeam(names(&["a"]))), vec![4, 1, 1, b'a']);
		assert_eq!(ser(&UpdateTeamOptions::CreateTeam(CreateTeam::mc_default()))[0], 0);
		assert_eq!(ser(&UpdateTeamOptions::UpdateTeamInfo(UpdateTeamInfo::mc_default()))[0], 2);
	}

	#[test]
	fn team_options_round_trip() {
		round_trip(UpdateTeamOptions::RemoveTeam);
		round_trip(UpdateTeamOptions::AddEntitiesToTeam(names(&["Steve", "Alex"])));
		round_trip(UpdateTeamOptions::RemoveEntitiesFromTeam(names(&[])));
		round_trip(UpdateTeamOptions::CreateTeam(CreateTeam {
			team_display_name: TextComponent::new("Red"),
			friendly_flags: FriendlyFlags::new(true, false),
			name_tag_visibility: NameTagVisibility::HideForOtherTeams,
			collision_rule: CollisionRule::PushOwnTeam,
			team_color: TeamColor::Red,
			team_prefix: TextComponent::new("[R] "),
			team_suffix: TextComponent::new(""),
			entities: names(&["Steve"]),
		}));
		round_trip(UpdateTeamOptions::UpdateTeamInfo(UpdateTeamInfo {
			team_display_name: TextComponent::new("Blue"),
			friendly_flags: FriendlyFlags::new(false, true),
			name_tag_visibility: NameTagVisibility::Never,
			collision_rule: CollisionRule::Never,
			team_color: TeamColor::Reset,
			team_prefix: TextComponent::new(""),
			team_suffix: TextComponent::new(""),
		}));
	}

	#[test]
	fn unknown_team_method_is_rejected() {
		let mut d = McDeserializer::new(&[5]);
		assert!(UpdateTeamOptions::mc_deserialize(&mut d).is_err());
	}

	#[test]
	fn friendly_flag_bits_match_wiki() {
		// Wiki (Update Teams): 0x01 allow friendly fire, 0x02 can see invisible teammates.
		assert_eq!(ser(&FriendlyFlags::new(true, false)), vec![0x01]);
		assert_eq!(ser(&FriendlyFlags::new(false, true)), vec![0x02]);
		let f = FriendlyFlags::mc_deserialize(&mut McDeserializer::new(&[0x03])).unwrap();
		assert!(f.allow_friendly_fire() && f.can_see_invisble_teammates());
	}

	#[test]
	fn enum_ids_match_wiki() {
		assert_eq!(ser(&NameTagVisibility::HideForOwnTeams), vec![3]);
		assert_eq!(ser(&CollisionRule::PushOtherTeams), vec![2]);
		assert_eq!(ser(&TeamColor::White), vec![15]);
		assert_eq!(ser(&TeamColor::Reset), vec![21]);
		assert_eq!(ser(&ObjectiveType::Hearts), vec![1]);
	}

	#[test]
	fn number_formats_round_trip() {
		assert_eq!(ser(&ObjectiveNumberFormat::Blank), vec![0]);
		round_trip(ObjectiveNumberFormat::Fixed(TextComponent::new("10")));
		assert_eq!(ser(&UpdateScoreFormat::Blank), vec![0]);
		round_trip(UpdateScoreFormat::Fixed(TextComponent::new("10")));
	}
}
