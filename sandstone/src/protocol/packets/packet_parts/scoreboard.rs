use crate::bitflag;
use crate::protocol::serialization::serializer_error::SerializingErr;
use crate::protocol::serialization::serializer_types::PrefixedArray;
use crate::protocol::serialization::McDeserialize;
use crate::protocol::serialization::McDeserializer;
use crate::protocol::serialization::McSerialize;
use crate::protocol::serialization::McSerializer;
use crate::protocol::serialization::SerializingResult;
use crate::protocol::testing::McDefault;
use crate::protocol_types::datatypes::chat::TextComponent;
use crate::protocol_types::datatypes::nbt::nbt::NbtCompound;
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
	Fixed(TextComponent) = 2
}
